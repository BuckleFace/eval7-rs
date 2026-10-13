# eval7-rs

[![CI](https://github.com/BuckleFace/eval7-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/BuckleFace/eval7-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/eval7-rs.svg)](https://crates.io/crates/eval7-rs)

`eval7-rs` is an exact seven-card poker hand evaluator for Rust. It maps a
validated hand to a compact, directly comparable rank using suit
canonicalisation, formula indexing, and an embedded lookup table.

The crate evaluates standard 52-card high poker. It has no runtime
dependencies, filesystem access, unsafe code, or table generation. It does
not provide five-card evaluation, equity calculation, ranges, game-state
logic, or a poker solver.

## Installation

```toml
[dependencies]
eval7-rs = "0.1.0"
```

The minimum supported Rust version is 1.85.

## Usage

```rust
use eval7_rs::{Card, HandCategory, evaluate_7};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let straight_flush = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
        .map(|text| text.parse::<Card>().expect("valid card"));
    let pair = ["Ac", "Ad", "Kh", "9s", "7c", "4d", "2h"]
        .map(|text| text.parse::<Card>().expect("valid card"));

    let stronger = evaluate_7(straight_flush)?;
    let weaker = evaluate_7(pair)?;

    assert_eq!(stronger.category(), HandCategory::StraightFlush);
    assert!(stronger > weaker);
    Ok(())
}
```

Cards can also be constructed from an ID in `0..=51`, or from a rank in
`2..=14` and suit in `0..=3`. IDs are suit-major in clubs, diamonds, hearts,
spades order, with ranks from deuce through ace.

`evaluate_7` rejects duplicate cards. If uniqueness is already guaranteed,
`evaluate_7_unchecked` skips only that check:

```rust
use eval7_rs::{Card, evaluate_7_unchecked};

let cards = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
    .map(|text| text.parse::<Card>().unwrap());
let rank = evaluate_7_unchecked(cards);
```

Despite its name, the unchecked function is safe Rust and performs no
unchecked memory access. Duplicate input may return a meaningless rank or
panic, but cannot cause undefined behaviour. Reuse an `Evaluator` when
evaluating many hands to initialize the shared formula index before the hot
loop.

## Rank semantics

`HandRank` implements `Ord`: larger values are stronger, and equality means
the best five-card hands tie. Categories are ordered as follows:

```text
High card < Pair < Two pair < Trips < Straight < Flush
          < Full house < Quads < Straight flush
```

`HandRank::raw()` returns the stable v1 packed `u16`; `category()` returns the
semantic category, and `payload()` returns its category-local ordering value.

## Implementation

Each card contributes a rank bit to one of four 13-bit suit masks. Sorting
those masks removes arbitrary suit names while preserving rank and flush
information. A combinatorial formula then maps the canonical 52-bit key to a
dense index, which selects one little-endian `u16` from the embedded table.

Suit-equivalent hands share an entry, reducing 133,784,560 physical hands to
6,009,159 canonical ranks. The versioned table is 12,018,382 bytes: a 64-byte
header plus 6,009,159 rank values. On first use, the formula indexer builds
about 1.26 MiB of support tables on the heap. See [data/README.md](data/README.md)
for the binary layout and checksums.

## Correctness

The embedded evaluator has been checked against the original direct evaluator
for all 133,784,560 distinct seven-card combinations, comparing complete raw
ranks, with zero mismatches. The exhaustive check remains reproducible as an
ignored release-mode test:

```text
cargo test --release exhaustive_all_seven_card_hands -- --ignored
```

Normal tests cover card validation, duplicate rejection, every hand category,
ordering, checked/unchecked equivalence, the embedded payload identity, and a
deterministic 50,000-hand comparison with both the direct evaluator and an
independent five-card-subset reference evaluator. The exhaustive test is kept
out of routine CI because it evaluates every possible hand.

## Performance

Local Criterion measurements on an AMD Ryzen 9 7950X3D with Windows 11 and
Rust 1.96.1 produced these point estimates:

| Workload | Checked | Prevalidated |
| --- | ---: | ---: |
| Repeated hand | 12.719 ns | 8.765 ns |
| 4,096 distributed hands | 13.735 ns | 10.504 ns |
| Fixed board, 1,081 hole pairs | 21.647 ns | 19.236 ns |
| Sequential 4,096-hand batch | 13.132 ns/hand | 10.070 ns/hand |

Criterion used 100 samples, a three-second warm-up, and a five-second
measurement period. Inputs passed through `black_box`; batch ranks were folded
into a black-boxed checksum. The machine was not isolated, CPU affinity was not
pinned, and these results are not cross-machine guarantees.

Run the standalone benchmark with:

```text
cargo bench --features bench-internals --bench evaluator
```

## Development

CI checks formatting, Clippy with warnings denied, default and all-feature
tests, rustdoc, the complete table checksum, and `cargo package`. Local
equivalents are defined in [the CI workflow](.github/workflows/ci.yml).

## Licence

Copyright (c) 2026 Jack Buckley.

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT licence](LICENSE-MIT), at your option.
