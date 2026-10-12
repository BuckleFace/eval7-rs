use std::hint::black_box;
use std::time::Instant;

use eval7_rs::benchmark_support::{
    EMBEDDED_TABLE_BYTES, FORMULA_INDEXER_HEAP_BYTES, FORMULA_INDEXER_INLINE_BYTES,
};
use eval7_rs::{Card, Evaluator};

fn main() {
    let hand = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
        .map(|card| card.parse::<Card>().expect("probe card should be valid"));

    let initialization_started = Instant::now();
    let evaluator = black_box(Evaluator::new());
    let initialization_ns = initialization_started.elapsed().as_nanos();

    let evaluation_started = Instant::now();
    let rank = evaluator.evaluate_7(black_box(hand)).unwrap();
    let first_evaluation_ns = evaluation_started.elapsed().as_nanos();

    println!(
        "{initialization_ns},{first_evaluation_ns},{},{EMBEDDED_TABLE_BYTES},{FORMULA_INDEXER_HEAP_BYTES},{FORMULA_INDEXER_INLINE_BYTES}",
        rank.raw()
    );
}
