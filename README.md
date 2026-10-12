# eval7-rs

High-performance seven-card poker hand evaluator in Rust using suit canonicalisation and precomputed lookup tables.

The rank table is embedded in the library. Evaluation does not access the filesystem or generate data at runtime.

```rust
use eval7_rs::{Card, HandCategory, evaluate_7};

let cards = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
    .map(|text| text.parse::<Card>().unwrap());
let rank = evaluate_7(cards).unwrap();

assert_eq!(rank.category(), HandCategory::StraightFlush);
```

All seven cards must be distinct. Invalid cards are rejected during construction, and duplicate cards are reported by `evaluate_7`.
