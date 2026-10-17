# eval7-rs

[![CI](https://github.com/BuckleFace/eval7-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/BuckleFace/eval7-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/eval7-rs.svg)](https://crates.io/crates/eval7-rs)

`eval7-rs` evaluates standard seven-card, 52-card high poker hands and returns
a compact, directly comparable rank. It has no runtime dependencies, allocation,
filesystem access, unsafe code or runtime table generation.

It does not provide five-card evaluation, equity calculation, ranges, game-state
logic or a poker solver. See [CHANGELOG.md](CHANGELOG.md) for release notes.

## Installation

```toml
[dependencies]
eval7-rs = "0.2.0"
```

The minimum supported Rust version is 1.85.

## Usage

```rust
use eval7_rs::{Card, HandCategory, evaluate_7};

let cards = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
    .map(|text| text.parse::<Card>().unwrap());
let rank = evaluate_7(cards).unwrap();

assert_eq!(rank.category(), HandCategory::StraightFlush);
```

Cards can also be constructed from an ID in `0..=51`, or a rank in `2..=14`
and suit in `0..=3`. IDs are suit-major: clubs, diamonds, hearts, spades,
with ranks from deuce through ace.

`evaluate_7` rejects duplicates. `evaluate_7_unchecked` skips that validation
for callers that already guarantee seven distinct cards. It is safe Rust:
duplicates may return a meaningless rank or panic, but cannot cause undefined
behaviour. The corresponding methods on `Evaluator` have the same semantics.
Evaluators are stateless and cheap to construct, with no first-use initialization.

For repeated evaluation against a fixed five-card board:

```rust
use eval7_rs::{Card, HandCategory, PreparedBoard};

let board = ["Ah", "Kh", "Qh", "2c", "3d"]
    .map(|text| text.parse::<Card>().unwrap());
let prepared = PreparedBoard::new(board).unwrap();
let holes = ["Jh", "Th"].map(|text| text.parse::<Card>().unwrap());
let rank = prepared.evaluate(holes).unwrap();
assert_eq!(rank.category(), HandCategory::StraightFlush);
```

Construction validates and aggregates the board once. Each evaluation validates
the two hole cards, including board overlap, and ranks the best five of all
seven cards. Neither hole card must be used. The immutable board is reusable,
including after errors, and performs no heap allocation.

## Rank semantics

`HandRank` implements `Ord`: larger values are stronger, and equality means
the best five-card hands tie. Categories are ordered as follows:

```text
High card < Pair < Two pair < Trips < Straight < Flush
          < Full house < Quads < Straight flush
```

`HandRank::raw()` retains the stable v1 packed `u16` encoding. `category()`
returns the category, and `payload()` its category-local ordering value.
The compact backend changes table addressing, not rank values or ordering.

## Implementation

Cards contribute to thirteen three-bit rank-count lanes and four suit counters.
Original suit masks are retained for flush lookups. If a suit has at least
five cards, its 13-bit mask selects the best flush or straight flush directly.
With only seven cards, a full house or quads cannot coexist with that flush.

Other hands use a dense index over 49,205 rank-count vectors. A carry-free
prefix sum and four three-rank lookups calculate the same ordinal as serial
rank-count indexing, without a per-rank dependency chain. The recurrence,
ordering and packing invariants are documented next to the code in `src/index.rs`.

Core data is 148,458 bytes (about 145 KiB): a 114,858-byte rank binary,
32 KiB of indexing support and 832 bytes of card contributions, excluding
code/alignment. All data is static; no heap indexer is initialized. See
[data/README.md](data/README.md) for the binary layout and identity checks.

## Correctness and performance

The ignored exhaustive test compares complete raw ranks with the unchanged
direct evaluator over all 133,784,560 distinct hands, including checked,
unchecked and prepared evaluation:

```text
cargo test --release exhaustive_all_seven_card_hands -- --ignored --nocapture
```

Routine tests also cover all 49,205 count-vector ordinals, every valid flush
mask in each suit, categories, wheels, duplicate rejection, board partitions,
suit permutations and 50,000 deterministic hands against both the direct
evaluator and an independent five-card-subset reference evaluator.

Run the public API benchmarks with:

```text
cargo bench --features bench-internals --bench evaluator
```

Compare the same fixtures, compiler settings, CPU placement and machine load.
Stage or initialization timings are not substitutes for complete evaluation
latency. Historical V1 measurements do not describe the compact backend.

## Development

CI checks formatting, Clippy with warnings denied, default/all-feature tests,
Rustdoc, the complete table checksum and `cargo package`. See
[the workflow](.github/workflows/ci.yml). Exhaustive verification is an explicit
release-mode check rather than part of routine CI.

## Licence

Copyright (c) 2026 Jack Buckley.

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT licence](LICENSE-MIT), at your option.
