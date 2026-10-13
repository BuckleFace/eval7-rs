# eval7-rs

[![CI](https://github.com/BuckleFace/eval7-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/BuckleFace/eval7-rs/actions/workflows/ci.yml)

`eval7-rs` is an exact seven-card poker hand evaluator for Rust. It converts a
validated seven-card hand into a compact, directly comparable rank using suit
canonicalisation, formula indexing, and an embedded lookup table.

The crate evaluates standard 52-card, high-hand poker rankings. It does not
provide five-card evaluation, hand parsing beyond individual cards, equity
calculation, ranges, game-state logic, or a poker solver.

Highlights:

- checked and prevalidated evaluation APIs;
- no runtime filesystem access or table generation;
- no runtime dependencies and no unsafe code;
- stable `u16` v1 rank encoding;
- exhaustive verification over all 133,784,560 distinct seven-card hands.

## Installation

Until the first crates.io release, depend on the GitHub repository:

```toml
[dependencies]
eval7-rs = { git = "https://github.com/BuckleFace/eval7-rs", branch = "main" }
```

The minimum supported Rust version is 1.85.

## Example

```rust
use eval7_rs::{Card, HandCategory, evaluate_7};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let straight_flush = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
        .map(|text| text.parse::<Card>().expect("valid card literal"));
    let pair = ["Ac", "Ad", "Kh", "9s", "7c", "4d", "2h"]
        .map(|text| text.parse::<Card>().expect("valid card literal"));

    let straight_flush_rank = evaluate_7(straight_flush)?;
    let pair_rank = evaluate_7(pair)?;

    assert_eq!(straight_flush_rank.category(), HandCategory::StraightFlush);
    assert!(straight_flush_rank > pair_rank);
    println!("packed rank: {}", straight_flush_rank.raw());
    Ok(())
}
```

Cards can also be constructed from a validated ID in `0..=51`, or from a rank
in `2..=14` and suit in `0..=3`. IDs are suit-major in clubs, diamonds,
hearts, spades order, with ranks from deuce through ace.

## Checked and prevalidated APIs

The normal interface verifies that all seven cards are distinct:

```rust
use eval7_rs::{Card, evaluate_7};

let cards = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
    .map(|text| text.parse::<Card>().unwrap());
let rank = evaluate_7(cards).unwrap();
```

For performance-sensitive code that has already established uniqueness,
`evaluate_7_unchecked` skips only the duplicate-card check:

```rust
use eval7_rs::{Card, evaluate_7_unchecked};

let cards = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
    .map(|text| text.parse::<Card>().unwrap());
let rank = evaluate_7_unchecked(cards);
```

Despite its name, this is a safe Rust function and performs no unchecked
memory access. Duplicate cards may produce a meaningless rank or panic; they
cannot cause undefined behaviour. Reuse an `Evaluator` when evaluating many
hands so the shared formula indexer is initialized before the hot loop.

## Rank semantics

`HandRank` implements `Ord`: larger values are stronger, and equality means
the best five-card poker hands tie exactly. Categories are ordered as:

```text
High card < Pair < Two pair < Trips < Straight < Flush
          < Full house < Quads < Straight flush
```

`HandRank::raw()` exposes the stable v1 packed `u16`; `category()` returns the
semantic category, and `payload()` returns its category-local ordering value.

## How it works

1. Each card contributes one rank bit to one of four 13-bit suit masks.
2. Sorting those masks removes the arbitrary names of the four suits while
   retaining all rank and flush information. The sorted masks form a
   canonical 52-bit key.
3. A combinatorial formula indexer maps every reachable canonical key to a
   dense index in `0..6_009_159` without storing the keys.
4. That index reads one little-endian `u16` from the embedded rank table.

Suit canonicalisation is the distinctive part of the design: hands that differ
only by a permutation of suit names share one table entry. This reduces
133,784,560 physical seven-card combinations to 6,009,159 canonical ranks.

The versioned table file is 12,018,382 bytes: a 64-byte header plus 6,009,159
rank values. It resides in read-only program data. On first use, the formula
indexer builds approximately 1.26 MiB of support tables on the heap; the local
baseline measured initialization at roughly 0.42 ms median. There is no
runtime file I/O.

See [the table format and conversion notes](data/README.md) for the binary
layout and checksums.

## Correctness

The release evaluator was checked against the ported direct evaluator for all
133,784,560 distinct seven-card combinations, comparing complete raw ranks
rather than categories alone. The exhaustive run reported zero mismatches.

Ordinary tests also cover card validation, duplicate rejection, every hand
category, category and kicker ordering, checked/unchecked equivalence, table
identity, and a deterministic 50,000-hand comparison with an independent
five-card-subset reference evaluator.

- [Exhaustive verification report](docs/verification/exhaustive-v1.md)
- Reproduce it with:

  ```text
  cargo run --release --manifest-path tools/exhaustive-verifier/Cargo.toml -- --report <report-path>
  ```

The exhaustive test is intentionally not part of normal CI.

## Performance

These are local Criterion measurements from an AMD Ryzen 9 7950X3D running
Windows 11 with Rust 1.96.1. They are not cross-machine guarantees. Criterion
used 100 samples, a three-second warm-up and a five-second measurement period;
the machine was not isolated and CPU affinity was not pinned.

| Workload | Checked mean | Prevalidated mean |
|---|---:|---:|
| Repeated hand | 12.719 ns | 8.765 ns |
| 4,096 distributed hands | 13.735 ns | 10.504 ns |
| Fixed board, 1,081 hole pairs | 21.647 ns | 19.236 ns |
| Sequential 4,096-hand batch | 13.132 ns/hand | 10.070 ns/hand |

Confidence intervals, raw methodology and caveats are in the
[checked/unchecked report](docs/benchmarks/unchecked-api-2026-10-12.md). Run
the repository benchmark with:

```text
cargo bench --features bench-internals --bench evaluator
```

The [external comparison report](docs/benchmarks/comprehensive-results-2026-10-12.md)
publishes both favourable and unfavourable results and explains where native
hand construction, validation and preprocessing make comparisons unequal. The
[architecture investigation](docs/benchmarks/algorithm-investigation-2026-10-12.md)
contains more implementation detail. This project does not claim to be the
fastest poker evaluator.

## Development tools

The repository retains three reproducibility tools that are excluded from the
published crate:

- `tools/table-converter`: converts and validates the legacy canonical table;
- `tools/exhaustive-verifier`: enumerates all seven-card combinations;
- `tools/comparison-benchmarks`: isolated external and diagnostic benchmarks.

Generated benchmark output belongs under `benchmark-results/` and is ignored
by Git.

## Licence

Copyright (c) 2026 Jack Buckley.

Licensed under either of
[Apache License, Version 2.0](LICENSE-APACHE) or [MIT licence](LICENSE-MIT) at
your option.
