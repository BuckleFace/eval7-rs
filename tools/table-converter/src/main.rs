use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

#[path = "../../../src/formula.rs"]
mod formula;

use formula::Canonical7FormulaIndexer;

const LEGACY_MAGIC: &[u8; 8] = b"PSCHT7R1";
const LEGACY_HEADER_BYTES: u64 = 16;
const LEGACY_ENTRY_BYTES: u64 = 10;

const OUTPUT_MAGIC: &[u8; 8] = b"E7RANKS\0";
const FORMAT_VERSION: u16 = 1;
const HEADER_BYTES: u16 = 64;
const ORDERING_ID: u16 = 1;
const RANK_ENCODING_ID: u16 = 1;
const ENTRY_WIDTH: u16 = 2;
const FLAGS_LITTLE_ENDIAN: u16 = 1;
const EXPECTED_ENTRY_COUNT: usize = 6_009_159;
const EXPECTED_PAYLOAD_SHA256: [u8; 32] = [
    0xce, 0x0e, 0xc3, 0xe9, 0xbe, 0x13, 0x32, 0xa7, 0x6d, 0x74, 0xcf, 0x47, 0x36, 0x8d, 0xd3, 0x79,
    0x53, 0xce, 0xf6, 0xe0, 0x97, 0xc3, 0x1c, 0xfb, 0x4a, 0x54, 0x06, 0x2c, 0x8b, 0x83, 0xb7, 0x74,
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let legacy_path = args.next().ok_or("missing legacy table path")?;
    let output_path = args.next().ok_or("missing output table path")?;
    if args.next().is_some() {
        return Err(format!(
            "usage: {} <canonical_hash_rank_7.bin> <eval7-ranks-v1.bin>",
            PathBuf::from(program).display()
        )
        .into());
    }

    let legacy_path = PathBuf::from(legacy_path);
    let output_path = PathBuf::from(output_path);
    let payload = convert_legacy_table(&legacy_path)?;
    let output = build_output_file(&payload);
    write_reproducibly(&output_path, &output)?;

    println!("validated canonical entries: {EXPECTED_ENTRY_COUNT}");
    println!("rank payload bytes: {}", payload.len());
    println!(
        "rank payload sha256: {}",
        hex_string(&EXPECTED_PAYLOAD_SHA256)
    );
    println!("output: {}", output_path.display());
    Ok(())
}

fn convert_legacy_table(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let file_len = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if &magic != LEGACY_MAGIC {
        return Err("invalid legacy table magic".into());
    }

    let mut count_bytes = [0u8; 8];
    reader.read_exact(&mut count_bytes)?;
    let count = u64::from_le_bytes(count_bytes);
    if count != EXPECTED_ENTRY_COUNT as u64 {
        return Err(format!("unexpected legacy entry count {count}").into());
    }

    let expected_len = LEGACY_HEADER_BYTES + count * LEGACY_ENTRY_BYTES;
    if file_len != expected_len {
        return Err(format!(
            "legacy file length {file_len} does not match expected {expected_len}"
        )
        .into());
    }

    let indexer = Canonical7FormulaIndexer::new();
    if indexer.total_count() != EXPECTED_ENTRY_COUNT {
        return Err("formula indexer count does not match expected entry count".into());
    }

    let mut payload = Vec::with_capacity(EXPECTED_ENTRY_COUNT * 2);
    let mut previous_key = None;
    for expected_index in 0..EXPECTED_ENTRY_COUNT {
        let mut entry = [0u8; LEGACY_ENTRY_BYTES as usize];
        reader.read_exact(&mut entry)?;
        let key = u64::from_le_bytes(entry[..8].try_into()?);
        let rank = u16::from_le_bytes(entry[8..].try_into()?);

        if previous_key.is_some_and(|previous| key <= previous) {
            return Err(format!(
                "legacy keys are not strictly increasing at entry {expected_index}"
            )
            .into());
        }
        previous_key = Some(key);

        let computed_index = indexer
            .try_index_for_key(key)
            .ok_or_else(|| format!("invalid canonical key {key} at entry {expected_index}"))?;
        if computed_index != expected_index {
            return Err(format!(
                "canonical key {key} at entry {expected_index} maps to {computed_index}"
            )
            .into());
        }

        if rank >> 12 > 8 {
            return Err(format!("invalid packed rank {rank} at entry {expected_index}").into());
        }
        payload.extend_from_slice(&rank.to_le_bytes());
    }

    let digest: [u8; 32] = Sha256::digest(&payload).into();
    if digest != EXPECTED_PAYLOAD_SHA256 {
        return Err(format!(
            "payload sha256 {} does not match expected {}",
            hex_string(&digest),
            hex_string(&EXPECTED_PAYLOAD_SHA256)
        )
        .into());
    }
    Ok(payload)
}

fn build_output_file(payload: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(HEADER_BYTES as usize + payload.len());
    output.extend_from_slice(OUTPUT_MAGIC);
    output.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    output.extend_from_slice(&HEADER_BYTES.to_le_bytes());
    output.extend_from_slice(&ORDERING_ID.to_le_bytes());
    output.extend_from_slice(&RANK_ENCODING_ID.to_le_bytes());
    output.extend_from_slice(&(EXPECTED_ENTRY_COUNT as u32).to_le_bytes());
    output.extend_from_slice(&ENTRY_WIDTH.to_le_bytes());
    output.extend_from_slice(&FLAGS_LITTLE_ENDIAN.to_le_bytes());
    output.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    output.extend_from_slice(&EXPECTED_PAYLOAD_SHA256);
    assert_eq!(output.len(), HEADER_BYTES as usize);
    output.extend_from_slice(payload);
    output
}

fn write_reproducibly(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        if fs::read(path)? == bytes {
            return Ok(());
        }
        return Err(format!(
            "refusing to replace different existing file {}",
            path.display()
        )
        .into());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(bytes)?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    drop(writer);
    fs::rename(&temporary, path)?;
    if fs::read(path)? != bytes {
        return Err(format!("written output failed verification: {}", path.display()).into());
    }
    Ok(())
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
