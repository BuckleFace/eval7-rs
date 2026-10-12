# Embedded rank table

`eval7-ranks-v1.bin` contains 6,009,159 formula-ordered little-endian `u16` ranks.

The 64-byte header contains:

- magic `E7RANKS\0`
- format version `1`
- header length `64`
- canonical ordering ID `1`
- rank encoding ID `1`
- entry count `6,009,159`
- entry width `2`
- flags `1` (little-endian payload)
- payload length `12,018,318`
- the 32-byte SHA-256 payload identity

The payload SHA-256 is:

```text
ce0ec3e9be1332a76d74cf47368dd37953cef6e097c31cfb4a54062c8b83b774
```

The complete file SHA-256 is:

```text
b2b2e1a4dbec03823d5bc7845158c6c771345536221a5035f3181ed3f3b66b59
```

Reproduce it from the legacy solver table with:

```text
cargo run --release --manifest-path tools/table-converter/Cargo.toml -- <legacy-table> data/eval7-ranks-v1.bin
```

The converter validates the legacy header, exact file length, strict key ordering, every canonical key, every formula index, packed rank categories, and the expected payload SHA-256 before writing the output.
