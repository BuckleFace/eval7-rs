# Comprehensive comparison benchmarks

This standalone, non-publishable Cargo package keeps comparison-only dependencies out of `eval7-rs`. It prepares performance comparisons against:

- `rs_poker = 5.0.0` with default features disabled;
- `aya_poker = 0.1.0` with default features disabled; and
- `poker = 0.7.0` with default features disabled.

All versions are exact pins in `Cargo.toml` and resolved by the committed package lockfile. `rs_poker 5.1.0` was rejected because its published source uses `u64::isolate_lowest_one`, which is unstable on Rust 1.96.1. `rust_poker 0.1.14` was rejected because its crate root enables the nightly-only `test` feature. No crates.io package named `poker-hand-evaluator` was found on 2026-10-12.

## Prepared workloads

- `random`: 64, 4,096, and 262,144 preconverted hands, with checked and unchecked Eval7 paths.
- `repeated_pool`: pools of 1, 8, 64, and 1,024 hands.
- `fixed_board`: dry, paired, monotone, and connected boards, each with all 1,081 legal hole-card pairs. The `rs_poker_shared_prefix` case is explicitly labelled because it uses that crate's solver-oriented accumulator API rather than full-hand input.
- `sequential_batch`: 4,096 and 262,144 pre-generated hands.
- `api_overhead`: duplicate validation, free-function versus reusable-evaluator calls, per-card conversion, and full native-representation conversion.
- `probe`: fresh-process initialization, first use, a short warm sequence, and process-memory observation.

Eval7's exact static accounting is reported alongside the process observations: 12,018,382 embedded bytes (64-byte header plus 6,009,159 little-endian `u16` ranks) and 1,319,072 requested heap bytes for a formula indexer, excluding allocator metadata. The inline indexer size is obtained from `size_of` for the current target. Competitor allocation totals are not exposed by their APIs, so those remain process-level comparisons against the `noop` probe.

The fixture generator maps an affine sequence onto lexicographic combination ranks. Its stride, 1,000,003, is coprime with C(52,7), so it visits every combination exactly once before repeating. Cards within each hand are strictly increasing. Generation and conversion happen outside Criterion timing except in the explicitly named conversion benchmarks.

## Running later

Do not run while the machine has interfering workloads. From the repository root, the full command is:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tools\comparison-benchmarks\run.ps1
```

To pin all child processes to a caller-selected set of logical processors:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tools\comparison-benchmarks\run.ps1 -AffinityMask 0x0000000F
```

The mask above is only an example. Verify the Windows logical-processor-to-CCD mapping on the actual machine before choosing it. The runner changes only its own process affinity and restores it; children inherit the mask. It does not change firmware, power plans, Game Mode, core parking, or other system-wide settings. Windows processor groups may limit this mechanism on machines with more than 64 logical processors.

A lightweight preflight-only invocation is available:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tools\comparison-benchmarks\run.ps1 -ValidateOnly
```

The full runner performs correctness preflight before timing, records system and Git metadata, runs Criterion in a fixed order, retains raw Criterion data and logs, launches fresh-process probes, captures approximate Windows process memory after touching 20,000 deterministic hands, and writes CSV and Markdown summaries under the ignored `benchmark-results/<timestamp>/` directory.

Criterion uses Cargo's `bench` profile and the probes use `release`; both set thin LTO and one codegen unit. The runner does not force `target-cpu=native`, so the default command represents normal generic-target compilation. Any caller-provided `RUSTFLAGS` are preserved and recorded; keep them identical between runs.

## Fairness limits

The timed steady-state comparisons use each crate's native prebuilt hand representation. Conversion is excluded and measured separately. Eval7's checked API and `poker` perform duplicate validation during evaluation; Eval7 unchecked, `rs_poker`, and `aya_poker` consume already-valid fixtures. These are deliberately separate API-level results, not claims that all APIs perform identical work.

Rank encodings differ. The preflight therefore compares semantic categories and complete ordering (including ties) rather than raw integers. It checks 20,000 deterministic hands, 19,999 adjacent rank comparisons, and a deliberate cross-suit tie. This is a compatibility gate, not exhaustive verification of competitor crates.

`poker` evaluates seven cards by selecting the best five-card subset and dynamically builds its lookup maps. `aya_poker` uses compile-time generated perfect-hash tables. `rs_poker 5.0.0` uses generated perfect-hash tables and also exposes a shared-prefix accumulator. Those architectural differences are part of what the suite measures and must be retained in interpretation.

The exact dependency licenses are Apache-2.0 for `rs_poker`, Zlib OR Apache-2.0 OR MIT for `aya_poker`, and MIT for `poker`. They are benchmark-tool dependencies only.
