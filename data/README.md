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

The table was converted once from the legacy canonical table for the v1
format. That legacy input is not distributed with this repository, so the
one-off converter is not retained. The library validates the format and
payload identity at compile time, tests the payload digest, and CI checks the
complete file digest. The v1 table must not be regenerated or modified.
