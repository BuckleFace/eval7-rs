# Changelog

## 0.2.0 (unreleased)

- Replace the suit-canonical V1 backend with compact rank-count and flush
  lookups. Core lookup/support data is now 148,458 bytes (about 145 KiB),
  including a 114,858-byte rank binary instead of the former 12,018,382-byte
  binary. Evaluation requires no heap allocation or runtime table generation.
- Add `PreparedBoard` for checked evaluation of two hole cards against a
  validated, reusable five-card board.
- Preserve existing evaluation signatures, duplicate rejection and the exact
  `HandRank` encoding and ordering. The unchecked API remains safe Rust and
  requires distinct cards.
- Verify checked, unchecked and prepared-board results against the direct
  evaluator over all 133,784,560 distinct seven-card hands, with no mismatches.

Local matched benchmarks improved across all measured ordinary-evaluation
fixtures. Measurements and their limitations are recorded in
[the backend PR](https://github.com/BuckleFace/eval7-rs/pull/5); timings depend on
the workload, compiler and machine. The minimum supported Rust version remains
1.85, with no runtime dependencies.
