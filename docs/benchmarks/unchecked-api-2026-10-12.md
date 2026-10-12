# Public unchecked API benchmark

This report measures the public checked and unchecked evaluation APIs after
adding `evaluate_7_unchecked`. The starting implementation is commit
`2054e88dda2c824a12990db930bec5ab5597a431`; the API and benchmark changes were
uncommitted when this run was made.

## Environment and methodology

The environment and corpus construction are identical to
[`baseline-2054e88.md`](baseline-2054e88.md):

- AMD Ryzen 9 7950X3D, Windows 11 Pro build 26200.
- `rustc 1.96.1`, `x86_64-pc-windows-msvc`, LLVM 22.1.2.
- Criterion 0.8.2 and Cargo's default optimized `bench` profile.
- 3-second warm-up, 5-second measurement, and 100 samples.
- No CPU affinity, machine isolation, custom `RUSTFLAGS`, or profile overrides.
- Identical hands are supplied to the checked and unchecked methods.
- Inputs pass through `std::hint::black_box`; batch results feed a black-boxed
  checksum.

Intervals below are Criterion's reported 95% confidence intervals. Throughput
intervals are the inverse of the corresponding time interval. Relative speedup
is the ratio of the two point estimates; it does not have a separately computed
confidence interval.

## Results

| Workload | Checked ns/eval (95% CI) | Checked Meval/s (95% CI) | Unchecked ns/eval (95% CI) | Unchecked Meval/s (95% CI) | Speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| Repeated hand | 12.719 (12.617–12.839) | 78.623 (77.888–79.258) | 8.765 (8.672–8.895) | 114.097 (112.425–115.311) | 1.451× |
| Random distributed | 13.735 (13.623–13.875) | 72.807 (72.072–73.405) | 10.504 (10.411–10.595) | 95.202 (94.384–96.052) | 1.308× |
| Fixed board | 21.647 (21.508–21.803) | 46.196 (45.865–46.494) | 19.236 (19.159–19.316) | 51.986 (51.771–52.195) | 1.125× |
| Sequential 4,096-hand batch | 13.132 (13.044–13.233) | 76.153 (75.570–76.666) | 10.070 (9.979–10.191) | 99.304 (98.124–100.210) | 1.304× |

The directly measured batch intervals were 53.427–54.201 µs checked and
40.876–41.743 µs unchecked. The separate valid-hand uniqueness check measured
2.362 ns (95% CI 2.323–2.432 ns).

## Comparison with commit `2054e88`

| Workload | Previous checked ns | Current checked ns | Previous internal unchecked ns | Current public unchecked ns |
| --- | ---: | ---: | ---: | ---: |
| Repeated hand | 12.778 | 12.719 | 9.213 | 8.765 |
| Random distributed | 14.311 | 13.735 | 11.005 | 10.504 |
| Fixed board | 21.775 | 21.647 | 19.913 | 19.236 |
| Sequential batch | 13.378 | 13.132 | 10.812 | 10.070 |

Criterion compared the unchanged benchmark names directly with the prior run.
It detected no checked-path performance change for repeated and fixed-board
hands. Random and batch checked results moved modestly faster but remained
within Criterion's configured noise threshold. The unchecked benchmark name
changed from `internal_unchecked` to `public_unchecked`, so Criterion did not
produce a direct statistical comparison for that column. Differences between
those separate runs must not be attributed solely to the API change.

Formula indexer construction-and-drop measured 487.94 µs (95% CI
483.48–492.75 µs). That result is ancillary to the API comparison.

## Reproduction

```powershell
cargo bench --features bench-internals --bench evaluator
```

Criterion output is generated under `target/criterion/` and remains ignored by
Git. The checked and unchecked benchmarks call the two public `Evaluator`
methods directly; the `bench-internals` feature is now needed only for isolated
validation and formula-indexer initialization measurements.
