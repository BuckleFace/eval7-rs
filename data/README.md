# Embedded compact rank table

`eval7-compact-v2.bin` is the unchanged compact table recovered from the
historical evaluator experiment. It replaces the large V1 canonical table;
the stored `HandRank` encoding and ordering are still v1.

All integer fields are little-endian. The 64-byte header contains:

| Offset | Field | Value |
| ---: | --- | --- |
| 0 | Eight-byte magic | `EV7CMP02` |
| 8 | Format version, u32 | 2 |
| 12 | Header bytes, u32 | 64 |
| 16 | Non-flush entries, u32 | 49,205 |
| 20 | Flush slots, u32 | 8,192 |
| 24 | Payload bytes, u32 | 114,794 |
| 28 | Rank encoding ID, u32 | 1 |
| 32 | Payload SHA-256 | 32 bytes |

The payload begins with 49,205 `u16` ranks ordered lexicographically by the
thirteen rank multiplicities (deuce first), each in 0..=4 with total seven.
`src/index.rs` documents how the blocked index preserves those ordinals.

The following 8,192 `u16` slots are addressed directly by a 13-bit suit mask.
Valid flush lookups have five, six or seven set bits; the other slots are not
used for distinct seven-card inputs. Entries already select the best flush
or straight flush, including the wheel. The complete file is 114,858 bytes.

Payload SHA-256:

```text
fa7d5f1842e324ddfd12e40f1b74b0e7bc4002159caaeb58abaa863be9168b72
```

Complete file SHA-256:

```text
e182e92ac13c7356fcf9e7315dc4b5822bea0731a7cdbdd35d5b82077f75b6ab
```

The library validates the header and recorded identity at compile time,
unit tests hash the actual payload, and CI hashes the complete file. Keep this
binary unchanged; construction of the 32 KiB indexing support table happens
at compile time and does not regenerate the rank payload.
