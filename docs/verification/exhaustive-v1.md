# Eval7 exhaustive verification report

- Result: **PASS**
- Seven-card hands evaluated: `133784560`
- Expected hand count: `133784560`
- Mismatches: `0`
- Elapsed seconds: `6.559037`
- Platform: `windows / x86_64 / windows`
- Source commit: `2506884c282f00c7011a41114b55c20d21cd059f`
- Worktree dirty: `true`
- Run Unix timestamp: `1791813320`
- Embedded payload SHA-256: `ce0ec3e9be1332a76d74cf47368dd37953cef6e097c31cfb4a54062c8b83b774`

## Toolchain

```text
rustc 1.96.1 (31fca3adb 2026-06-26)
binary: rustc
commit-hash: 31fca3adb283cc9dfd56b49cdee9a96eb9c96ffd
commit-date: 2026-06-26
host: x86_64-pc-windows-msvc
release: 1.96.1
LLVM version: 22.1.2
```

## Reproduction

```powershell
cargo run --release --manifest-path tools/exhaustive-verifier/Cargo.toml -- --report docs/verification/exhaustive-v1.md
```

## Method

The release-mode verifier enumerates every combination `0 <= a < b < c < d < e < f < g < 52` exactly once. For each combination it compares the complete raw `u16` rank returned by the public embedded-table `Evaluator` with the complete raw `u16` rank returned by the ported direct seven-card evaluator. It stops at the first mismatch.
