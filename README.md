# eval7-rs

High-performance seven-card poker hand evaluator in Rust using suit canonicalisation and precomputed lookup tables.

The rank table is embedded in the library. Evaluation does not access the filesystem or generate data at runtime.

## Checked evaluation

```rust
use eval7_rs::{Card, HandCategory, evaluate_7};

let cards = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
    .map(|text| text.parse::<Card>().unwrap());
let rank = evaluate_7(cards).unwrap();

assert_eq!(rank.category(), HandCategory::StraightFlush);
```

Invalid cards are rejected during construction, and duplicate cards are reported
by `evaluate_7`.

## Unchecked evaluation

```rust
use eval7_rs::{Card, HandCategory, evaluate_7_unchecked};

let cards = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
    .map(|text| text.parse::<Card>().unwrap());
let rank = evaluate_7_unchecked(cards);

assert_eq!(rank.category(), HandCategory::StraightFlush);
```

`evaluate_7_unchecked` skips only the distinct-card check. The caller must
supply seven unique cards. Duplicate cards may produce a meaningless result or
panic, but cannot cause undefined behavior. This interface is intended for
performance-sensitive code that has already established hand validity.
