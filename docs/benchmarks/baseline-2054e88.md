# Eval7 performance baseline

This report records the unoptimized evaluator baseline for source commit
`2054e88dda2c824a12990db930bec5ab5597a431`. The benchmark instrumentation was
added after that commit and was uncommitted when these measurements were taken.

## Environment

- Date: 2026-10-12
- CPU: AMD Ryzen 9 7950X3D 16-Core Processor (16 cores, 32 logical processors)
- Memory: 67,772,514,304 bytes installed (approximately 63.1 GiB)
- OS: Microsoft Windows 11 Pro, version `10.0.26200`, 64-bit, build `26200`
- Rust: `rustc 1.96.1 (31fca3adb 2026-06-26)`
- Cargo: `cargo 1.96.1 (356927216 2026-06-26)`
- Target: `x86_64-pc-windows-msvc`
- LLVM: `22.1.2`
- Criterion: `0.8.2`, Plotters backend
- Compilation: Cargo's default optimized `bench` profile; no `RUSTFLAGS` or
  Cargo profile environment overrides were set. This means optimization level
  3, debug assertions and overflow checks disabled, LTO disabled, and 16
  code-generation units under Cargo's default profile settings.

The machine was not isolated, CPU affinity was not pinned, and background
activity and dynamic clock behavior were not controlled. These results are a
local baseline, not a cross-machine performance claim.

## Criterion results

Criterion used a 3-second warm-up, a 5-second measurement period, and 100
samples per benchmark. Time ranges are Criterion's reported 95% confidence
intervals. Throughput intervals are the inverse of the time interval; the
middle value is the inverse of Criterion's point estimate.

| Workload | Path | Time per evaluation, ns (95% CI) | Million evaluations/s (95% CI) |
| --- | --- | ---: | ---: |
| Repeated same hand | Public checked | 12.778 (12.573–13.032) | 78.260 (76.734–79.536) |
| Repeated same hand | Internal unchecked | 9.213 (9.056–9.390) | 108.540 (106.492–110.423) |
| Repeated same hand | Validation only | 2.429 (2.365–2.507) | 411.726 (398.819–422.833) |
| Random distributed | Public checked | 14.311 (14.064–14.613) | 69.876 (68.432–71.104) |
| Random distributed | Internal unchecked | 11.005 (10.887–11.151) | 90.868 (89.678–91.853) |
| Fixed board | Public checked | 21.775 (21.652–21.921) | 45.924 (45.618–46.185) |
| Fixed board | Internal unchecked | 19.913 (19.837–19.992) | 50.218 (50.020–50.411) |
| Sequential 4,096-hand batch | Public checked | 13.378 (13.305–13.452) | 74.749 (74.339–75.161) |
| Sequential 4,096-hand batch | Internal unchecked | 10.812 (10.621–11.041) | 92.490 (90.571–94.151) |

The batch's directly measured whole-batch intervals were 54.496–55.099 µs for
the checked path and 43.505–45.224 µs for the unchecked path. Criterion
reported high-side outliers in several tests (4–14% depending on the test);
none were manually removed.

The valid-hand duplicate validation step costs approximately 2.429 ns in
isolation on the repeated input. The difference between checked and unchecked
paths also includes normal measurement and code-layout effects, so it should
not be inferred solely by subtracting point estimates.

## Initialization

The repeatable Criterion initialization benchmark constructs and drops a fresh
`Canonical7FormulaIndexer` on every iteration. It measures 492.33 µs, with a
95% confidence interval of 489.78–494.97 µs. This includes allocation,
support-table construction, and deallocation. It does not measure `OnceLock`
contention.

A separate release-mode probe measured the real first `Evaluator::new()` in
100 fresh processes. Results were:

| Measurement | Minimum | Median | Mean (approx. 95% CI) | Maximum |
| --- | ---: | ---: | ---: | ---: |
| First `Evaluator::new()` | 377.4 µs | 421.6 µs | 429.4 µs (422.4–436.5) | 586.8 µs |
| First evaluation after initialization | 1.000 µs | 1.250 µs | 1.302 µs (1.254–1.350) | 2.700 µs |

The probe confidence intervals are conventional t-intervals over independent
process observations, not Criterion bootstrap intervals. The first evaluation
numbers include cold code and data-page effects and should not be compared to
the steady-state nanosecond results as though they measured the same thing.

## Memory accounting

The embedded table is exactly 12,018,382 bytes (11.462 MiB): a 64-byte header
and a 12,018,318-byte rank payload. It is compiled into the executable's
read-only data rather than allocated on the heap. The operating system may
demand-page that mapping, so its resident physical-memory contribution depends
on which table pages have been touched.

One formula indexer requests 1,319,072 heap bytes (1.258 MiB) in two boxed
allocations:

- 8,192 bytes for mask popcounts.
- 1,310,880 bytes for `5 × 8,193` rows of eight `u32` tail counts.

On this 64-bit target, the indexer itself occupies another 40 inline bytes in
the static `OnceLock`. The heap figure excludes allocator metadata and page
rounding. Embedded table bytes plus requested indexer heap total approximately
12.720 MiB, before code, allocator overhead, stacks, and other process state.

## Methodology

- The repeated hand is `As Ks Qs Js Ts 2d 3c`.
- The random corpus contains 4,096 deterministic valid hands generated with a
  fixed-seed LCG and duplicate rejection. Corpus generation is outside the
  timed loop.
- The fixed-board corpus uses `As Kd 7h 4c 2s` and enumerates all 1,081 pairs
  of remaining hole cards. Corpus generation is outside the timed loop.
- Sequential throughput evaluates the entire 4,096-hand random corpus per
  Criterion iteration and combines raw ranks into a checksum.
- The public path calls `Evaluator::evaluate_7`, including duplicate checking.
- The internal path uses a feature-gated, documentation-hidden wrapper around
  the same canonicalization, formula index, and embedded rank lookup, omitting
  only duplicate validation. All benchmark inputs on this path are distinct.
- Validation-only timing calls the production validation routine through the
  same benchmark support surface.
- Inputs are passed through `std::hint::black_box`; batch ranks are accumulated
  into a black-boxed checksum so the compiler cannot remove evaluation work.
- No evaluator algorithm, canonical indexing formula, rank encoding, or table
  data was changed for these measurements.

## Reproduction

Run the statistical benchmark suite:

```powershell
cargo bench --features bench-internals --bench evaluator
```

Criterion's generated raw estimates and HTML report are placed under
`target/criterion/` and are intentionally ignored by Git.

Build the fresh-process probe:

```powershell
cargo build --release --example first_use --features bench-internals
```

Collect 100 first-use observations. Each output row is
`initialization_ns,first_evaluation_ns,rank,embedded_bytes,indexer_heap_bytes,indexer_inline_bytes`:

```powershell
1..100 | ForEach-Object { & .\target\release\examples\first_use.exe }
```

The benchmark feature is disabled by default and exists only to expose the
unchecked and initialization paths to repository-owned measurement tools.
