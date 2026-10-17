use crate::HandRank;

// File layout (all integers little-endian): a 64-byte header followed by one
// `u16` HandRank for each canonical key in formula-index order. The header
// stores the magic, format and encoding IDs, dimensions, flags, and payload
// SHA-256; data/README.md records their concrete values.
pub(crate) const ENTRY_COUNT: usize = 6_009_159;
const HEADER_BYTES: usize = 64;
const PAYLOAD_BYTES: usize = ENTRY_COUNT * 2;
const FILE_BYTES: usize = HEADER_BYTES + PAYLOAD_BYTES;
const TABLE_FILE: &[u8; FILE_BYTES] = include_bytes!("../data/eval7-ranks-v1.bin");

#[allow(dead_code)] // Used by const evaluation; Rust 1.85 does not count that as a lint use.
pub(crate) const EXPECTED_PAYLOAD_SHA256: [u8; 32] = [
    0xce, 0x0e, 0xc3, 0xe9, 0xbe, 0x13, 0x32, 0xa7, 0x6d, 0x74, 0xcf, 0x47, 0x36, 0x8d, 0xd3, 0x79,
    0x53, 0xce, 0xf6, 0xe0, 0x97, 0xc3, 0x1c, 0xfb, 0x4a, 0x54, 0x06, 0x2c, 0x8b, 0x83, 0xb7, 0x74,
];

const _: () = validate_header(TABLE_FILE);

/// Looks up the rank stored at a dense formula index.
///
/// The caller must supply an index below [`ENTRY_COUNT`]. Formula indexing and
/// payload ordering use the same canonical-key order, so no key is stored in
/// each entry.
#[inline(always)]
pub(crate) fn rank_at(index: usize) -> HandRank {
    debug_assert!(index < ENTRY_COUNT);
    let offset = HEADER_BYTES + index * 2;
    HandRank::from_raw(u16::from_le_bytes([
        TABLE_FILE[offset],
        TABLE_FILE[offset + 1],
    ]))
}

#[allow(dead_code)] // Called by the compile-time header assertion above.
const fn validate_header(bytes: &[u8; FILE_BYTES]) {
    // This function is evaluated by the anonymous constant above. A mismatched
    // or incompatible embedded file therefore fails the build rather than a
    // later evaluation. The recorded digest identifies the payload; its bytes
    // are not recomputed here and are independently checked by the unit test.
    let magic = b"E7RANKS\0";
    let mut i = 0;
    while i < magic.len() {
        assert!(bytes[i] == magic[i], "invalid embedded table magic");
        i += 1;
    }
    assert!(read_u16(bytes, 8) == 1, "unsupported table format");
    assert!(
        read_u16(bytes, 10) == HEADER_BYTES as u16,
        "invalid header length"
    );
    assert!(read_u16(bytes, 12) == 1, "unsupported canonical ordering");
    assert!(read_u16(bytes, 14) == 1, "unsupported rank encoding");
    assert!(
        read_u32(bytes, 16) == ENTRY_COUNT as u32,
        "invalid entry count"
    );
    assert!(read_u16(bytes, 20) == 2, "invalid entry width");
    assert!(read_u16(bytes, 22) == 1, "invalid table flags");
    assert!(
        read_u64(bytes, 24) == PAYLOAD_BYTES as u64,
        "invalid payload length"
    );

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
const fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
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

#[allow(dead_code)]
const fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn embedded_payload_has_expected_sha256() {
        let digest: [u8; 32] = Sha256::digest(&TABLE_FILE[HEADER_BYTES..]).into();
        assert_eq!(digest, EXPECTED_PAYLOAD_SHA256);
    }
}
