# End-to-end and internal diagnostic benchmark preparation

Status: prepared but not executed. This document contains no new performance results.

## Suites

The standalone comparison package now has a second Criterion target, `diagnostics`, independent of the original `comparison` target.

### Common raw-input workloads

`raw_end_to_end/{64|4096|262144}` starts every implementation from the same copied `[u8; 7]`. Card conversion, native state construction, per-hand preprocessing, and ranking all occur after the input is passed through `black_box`.

Variants are deliberately explicit:

| Case | Validation semantics inside timing |
|---|---|
| `eval7_checked` | Validates each ID while constructing `Card`s and checks seven-card distinctness in `evaluate_7`. |
| `eval7_prevalidated` | Validates each ID while constructing `Card`s; skips duplicate validation in `evaluate_7_unchecked`. Dataset generation guarantees uniqueness. |
| `aya_poker_checked` | Converts seven cards, accumulates them, then `Hand::extend` asserts count/mask consistency and the seven-card limit. |
| `aya_poker_prevalidated` | Converts and accumulates through `insert_unchecked`; dataset generation guarantees uniqueness and size. |
| `rs_poker_hand_checked` | Converts cards, inserts into `Hand`, and asserts every `insert` reports a new card; `Hand::rank` then builds its evaluation accumulator. |
| `rs_poker_accumulator_prevalidated` | Converts cards directly into `SevenCardAccum`; generated fixtures guarantee validity. |
| `poker_checked` | Converts seven cards and consumes the normal fallible evaluator result. |

The naming does not claim identical validation algorithms. It separates conventional checked/container paths from trusted-input paths and preserves each crate's public behavior.

`native_construction/{size}` performs the same conversions and native construction but black-boxes the representation instead of ranking it. `preconstructed_evaluation/{size}` starts from the resulting native types, matching the boundary of the original comparison and adding rs_poker's pre-accumulated representation.

The three datasets use the existing deterministic affine traversal of C(52,7). They contain 64, 4,096, and 262,144 unique hands without replacement. Every implementation within a group receives the same sequence.

### Fixed boards

For the existing dry, paired, monotone, and connected boards, both suites traverse all 1,081 legal hole pairs.

- `fixed_board_raw_full/{texture}` receives the complete raw seven-card IDs and rebuilds every native representation. Nobody reuses board preprocessing.
- `fixed_board_raw_incremental/{texture}` stores only the changing raw hole IDs in the timed traversal. The fixed board may be converted before timing. Aya copies a five-card `Hand`; rs_poker copies either a five-card `Hand` or `SevenCardAccum`; Eval7 and poker reuse converted board cards but their current APIs still evaluate a complete seven-card input.

The incremental group is a solver-capability comparison, not equal internal work. It remains separate from full rebuilding for that reason.

### Eval7 stages

`eval7_internal/{64|4096|262144}` isolates:

1. `suit_masks`: seven card-to-lane lookups and ORs, producing four unsorted 13-bit masks;
2. `sort_four_masks`: production `sort_unstable` on an already-produced `[u16; 4]`;
3. `pack_key_from_sorted_masks`: bit-packing already-sorted masks;
4. `canonical_key_complete`: integrated mask construction, sorting, and packing;
5. `formula_index`: the production shared formula indexer from an already-constructed key;
6. `rank_table_scattered`: production little-endian rank access from an already-calculated representative index;
7. `unchecked_complete`: the unchanged complete Eval7 path.

Inputs for each isolated stage are precomputed from the preceding stage before timing. A copied input is black-boxed on every iteration and the output is black-boxed. The feature-gated wrappers call the same internal functions used by production evaluation. They are available only with `bench-internals`.

The measurements are not algebraically additive. Inlining, common-subexpression elimination, register residency, bounds-check optimization, cache state, and instruction overlap differ between isolated and integrated code. Stage results diagnose likely contributors; they do not predict complete latency by summing means.

### Cache-locality diagnostics

`eval7_table_access/{64|4096|262144}` compares:

- contiguous dense table indices, traversed sequentially;
- formula indices from the same deterministic representative hands, producing scattered table accesses.

The working-set sizes are labelled small, medium, and large in analysis rather than asserting a particular cache level. Whether they fit a cache depends on the active 7950X3D CCD, competing data, associativity, and scheduler placement.

Formula-support locality is exercised separately by `eval7_internal/*/formula_index`. The canonical keys are precomputed; only formula-support lookup and index arithmetic are timed.

## Anti-optimization measures

- Raw IDs or isolated-stage inputs are loaded through `black_box` after a runtime cursor selects them.
- Native construction occurs inside the end-to-end and construction-only timed closures.
- Every rank or intermediate result is passed through `black_box`.
- Fixture arrays are runtime `Vec`s generated before Criterion sampling, not compile-time rank answers.
- Batch-independent setup is limited to reusable evaluator objects, immutable generated tables, and explicitly documented fixed-board state.

Thin LTO and normal inlining remain enabled because the goal is optimized public-API performance. The barriers prevent value hoisting; they do not artificially prevent legitimate inlining.

## Runner behavior

The normal runner now executes the original comparison target followed by diagnostics, then performs the existing probes. Diagnostics-only mode skips the original 77 Criterion cases and skips the existing initialization/memory probes while retaining:

- correctness preflight;
- exact Git revision and dirty-state metadata;
- CPU, OS, Cargo, rustc, RUSTFLAGS, profile, and affinity metadata;
- raw Criterion `new`, `base`, sample and change files;
- separate diagnostic console logging;
- de-duplicated CSV and Markdown summaries;
- existing error and partial-result preservation.

Exact diagnostics-only command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tools\comparison-benchmarks\run.ps1 -DiagnosticsOnly
```

Optional affinity:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tools\comparison-benchmarks\run.ps1 -DiagnosticsOnly -AffinityMask 0x0000000F
```

The mask shown is only an example. It must not be used until the machine's logical-processor-to-CCD mapping is verified.

## Hardware counters

This Windows host has AMD uProf CLI/PCM and Windows Performance Recorder/Analyzer installed. Linux `perf` is not available. No counter collection has been performed.

For a later controlled run, AMD uProf is the preferred candidate because it can expose AMD-family events and effective frequency. The desired event classes are:

- retired instructions and core cycles;
- L1 data-cache accesses/misses;
- L2 requests/misses;
- last-level-cache requests/misses;
- data-TLB misses and page walks;
- branch instructions/mispredictions;
- effective and reference frequency.

Before collection, use `AMDuProfCLI info` and the installed version's event-list/help commands to identify supported Zen 4 event names. Event syntax and permissions are version- and driver-dependent; commands must not be invented or copied from a different uProf release. Collect separate runs pinned to a verified V-cache CCD core and a verified non-V-cache CCD core. Save the raw uProf session alongside the Criterion run metadata, but do not mix counter-instrumented timing with the primary uninstrumented latency result.

WPR/WPA can record scheduling, context switches, CPU usage, frequency, hard faults, and system-level cache information where exposed, but it is not a substitute for confirmed model-specific cache/TLB counters. If the uProf driver or the requested events are unavailable at execution time, record that limitation and omit those metrics.

## Preparation validation

Preparation should run only:

```powershell
cargo fmt --all -- --check
cargo fmt --manifest-path .\tools\comparison-benchmarks\Cargo.toml --all -- --check
cargo test --locked --all-features
cargo test --locked --manifest-path .\tools\comparison-benchmarks\Cargo.toml
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo clippy --locked --manifest-path .\tools\comparison-benchmarks\Cargo.toml --all-targets -- -D warnings
cargo bench --locked --manifest-path .\tools\comparison-benchmarks\Cargo.toml --bench diagnostics --no-run
```

The final command compiles the Criterion executable but does not execute it. Do not use `cargo bench` without `--no-run` during preparation.
