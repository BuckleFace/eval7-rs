use crate::HandRank;
use crate::index::{FLUSH_ENTRIES, NON_FLUSH_ENTRIES};

// The payload starts with dense rank-count entries, followed by direct 13-bit
// flush-mask slots. Both contain the unchanged v1 HandRank encoding.
pub(crate) const ENTRY_COUNT: usize = NON_FLUSH_ENTRIES + FLUSH_ENTRIES;
const HEADER_BYTES: usize = 64;
const PAYLOAD_BYTES: usize = ENTRY_COUNT * 2;
const FILE_BYTES: usize = HEADER_BYTES + PAYLOAD_BYTES;
const TABLE_FILE: &[u8; FILE_BYTES] = include_bytes!("../data/eval7-compact-v2.bin");

#[allow(dead_code)] // Const-only uses are not counted by Rust 1.85's dead-code lint.
pub(crate) const EXPECTED_PAYLOAD_SHA256: [u8; 32] = [
    0xfa, 0x7d, 0x5f, 0x18, 0x42, 0xe3, 0x24, 0xdd, 0xfd, 0x12, 0xe4, 0x0f, 0x1b, 0x74, 0xb0, 0xe7,
    0xbc, 0x40, 0x02, 0x15, 0x9c, 0xaa, 0xeb, 0x58, 0xab, 0xaa, 0x86, 0x3b, 0xe9, 0x16, 0x8b, 0x72,
];

const _: () = validate_header(TABLE_FILE);

/// Reads a rank at a compact payload index below ENTRY_COUNT.
#[inline(always)]
pub(crate) fn rank_at(index: usize) -> HandRank {
    debug_assert!(index < ENTRY_COUNT);
    let offset = HEADER_BYTES + index * 2;
    HandRank::from_raw(u16::from_le_bytes([
        TABLE_FILE[offset],
        TABLE_FILE[offset + 1],
    ]))
}

#[allow(dead_code)]
const fn validate_header(bytes: &[u8; FILE_BYTES]) {
    // Build-time compatibility and identity checks, not a recomputed checksum.
    // Tests independently hash the actual payload; CI hashes the complete file.
    let magic = b"EV7CMP02";
    let mut i = 0;
    while i < magic.len() {
        assert!(bytes[i] == magic[i], "invalid compact table magic");
        i += 1;
    }
    assert!(read_u32(bytes, 8) == 2, "unsupported table format");
    assert!(
        read_u32(bytes, 12) == HEADER_BYTES as u32,
        "invalid header length"
    );
    assert!(
        read_u32(bytes, 16) == NON_FLUSH_ENTRIES as u32,
        "invalid count-vector dimension"
    );
    assert!(
        read_u32(bytes, 20) == FLUSH_ENTRIES as u32,
        "invalid flush dimension"
    );
    assert!(
        read_u32(bytes, 24) == PAYLOAD_BYTES as u32,
        "invalid payload length"
    );
    assert!(read_u32(bytes, 28) == 1, "unsupported rank encoding");
    i = 0;
    while i < EXPECTED_PAYLOAD_SHA256.len() {
        assert!(
            bytes[32 + i] == EXPECTED_PAYLOAD_SHA256[i],
            "unexpected payload identity"
        );
        i += 1;
    }
}

#[allow(dead_code)]
const fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn embedded_payload_has_expected_sha256() {
        let digest: [u8; 32] = Sha256::digest(&TABLE_FILE[HEADER_BYTES..]).into();
        assert_eq!(digest, EXPECTED_PAYLOAD_SHA256);
    }
}
