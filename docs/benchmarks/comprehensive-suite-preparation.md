# Comprehensive benchmark suite preparation

Prepared on 2026-10-12 from source revision `40042c3` plus the uncommitted benchmark-suite changes described here. No performance measurements were executed during preparation.

The suite lives in `tools/comparison-benchmarks` as an isolated Cargo package. See its README for workloads, exact dependency pins, fixture methodology, fairness qualifications, commands, and Ryzen 9 7950X3D affinity support.

## Measurement design

Criterion is configured for a 3-second warm-up, 5-second measurement window, 80 samples, and 95% confidence intervals. The Cargo `bench` and `release` profiles both use thin LTO and one codegen unit; `RUSTFLAGS` is preserved and recorded rather than overridden. The suite contains 77 Criterion cases:

- 15 random-distribution cases across three working-set sizes;
- 20 repeated/pool cases across four pool sizes;
- 24 fixed-board cases across four textures, including four separately labelled `rs_poker` shared-prefix cases;
- 10 large sequential-batch cases; and
- 8 API/conversion cases.

The total is intentionally a count of cases rather than a runtime promise. Criterion may extend sampling to satisfy its iteration model, compilation is machine-dependent, and the runner additionally performs 120 fresh-process initialization observations plus six process-memory snapshots.

Initialization is not mixed into steady-state Criterion results. The probe reports construction, first evaluation, and 1,000 warm evaluations from a fresh process. The runner records the raw observations instead of overstating their statistical quality. “Fresh process” does not imply a cold operating-system file cache. Process memory is captured through Windows `Get-Process`; it includes runtime and loader overhead, so the `noop` process is provided as a practical baseline. Exact Eval7 embedded-table and formula-index allocation sizes are printed by the probe.

## Reliability checklist

Before the authorized run:

1. Close CPU-, disk-, and memory-intensive applications and pause scheduled scans or updates where practical.
2. Connect stable cooling and power; use one unchanged Windows power mode for the entire run.
3. Let the machine reach a stable idle temperature, and avoid interacting with it during measurement.
4. Decide whether to leave Windows scheduling uncontrolled or supply a verified affinity mask. Do not infer CCD membership solely from mask position.
5. Record any BIOS, Curve Optimizer, EXPO, Game Mode, chipset-driver, or power-plan choices separately; the runner cannot discover all of them.
6. Keep the repository and Rust toolchain unchanged between comparison runs.
7. Inspect `raw/preflight.log` and `failures.txt` before interpreting results.
8. Treat process-memory differences as approximate and initialization samples as raw observations; use Criterion confidence intervals for steady-state comparisons.

The configured suite is substantial—dozens of Criterion cases spanning cache-resident and larger working sets, plus fresh-process probes. Plan for a quiet, multi-minute measurement window, but do not rely on a precise duration estimate.
