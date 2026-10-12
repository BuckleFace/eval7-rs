use std::env;
use std::hint::black_box;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

use eval7_comparison_benchmarks::{aya_poker_rank, native_hand, rs_poker_rank, sampled_card_id};
use eval7_rs::Evaluator as Eval7Evaluator;
use eval7_rs::benchmark_support::FreshFormulaIndexer;
use eval7_rs::benchmark_support::{
    EMBEDDED_TABLE_BYTES, FORMULA_INDEXER_HEAP_BYTES, FORMULA_INDEXER_INLINE_BYTES,
};

fn main() {
    let mut arguments = env::args().skip(1);
    let evaluator = arguments.next().unwrap_or_else(|| usage());
    let hold_ms = arguments
        .next()
        .map(|value| value.parse::<u64>().unwrap_or_else(|_| usage()))
        .unwrap_or(0);
    let touch_count = arguments
        .next()
        .map(|value| value.parse::<usize>().unwrap_or_else(|_| usage()))
        .unwrap_or(0);
    if arguments.next().is_some() {
        usage();
    }

    let hand = native_hand([0, 14, 28, 42, 8, 22, 36]);
    let initialization_started = Instant::now();
    let state = match evaluator.as_str() {
        "noop" => State::Noop,
        "eval7" => State::Eval7(Eval7Evaluator::new()),
        "eval7-formula" => State::Formula(FreshFormulaIndexer::new()),
        "rs_poker" | "aya_poker" => State::Noop,
        "poker" => State::Poker(poker::Evaluator::new()),
        _ => usage(),
    };
    let initialization_ns = initialization_started.elapsed().as_nanos();

    let first_started = Instant::now();
    evaluate_once(&evaluator, &state, &hand);
    let first_ns = first_started.elapsed().as_nanos();

    let warm_started = Instant::now();
    for _ in 0..1_000 {
        evaluate_once(&evaluator, &state, black_box(&hand));
    }
    let warm_1000_ns = warm_started.elapsed().as_nanos();

    for index in 0..touch_count {
        let touched_hand = native_hand(sampled_card_id(index));
        evaluate_once(&evaluator, &state, black_box(&touched_hand));
    }

    println!(
        "evaluator={evaluator} pid={} initialization_ns={initialization_ns} first_evaluation_ns={first_ns} warm_1000_ns={warm_1000_ns} embedded_table_bytes={EMBEDDED_TABLE_BYTES} formula_heap_bytes={FORMULA_INDEXER_HEAP_BYTES} formula_inline_bytes={FORMULA_INDEXER_INLINE_BYTES}",
        std::process::id(),
    );
    io::stdout().flush().expect("stdout should flush");
    thread::sleep(Duration::from_millis(hold_ms));
    black_box(state);
}

enum State {
    Noop,
    Eval7(Eval7Evaluator),
    Formula(FreshFormulaIndexer),
    Poker(poker::Evaluator),
}

fn evaluate_once(evaluator: &str, state: &State, hand: &eval7_comparison_benchmarks::NativeHand) {
    match (evaluator, state) {
        ("noop", _) => {}
        ("eval7-formula", State::Formula(indexer)) => {
            black_box(indexer.total_count());
        }
        ("eval7", State::Eval7(evaluator)) => {
            black_box(evaluator.evaluate_7_unchecked(hand.eval7));
        }
        ("rs_poker", _) => {
            black_box(rs_poker_rank(hand));
        }
        ("aya_poker", _) => {
            black_box(aya_poker_rank(hand));
        }
        ("poker", State::Poker(evaluator)) => {
            black_box(
                evaluator
                    .evaluate_five(hand.poker)
                    .expect("probe hand is valid"),
            );
        }
        _ => unreachable!("state and evaluator were constructed together"),
    }
}

fn usage() -> ! {
    eprintln!(
        "usage: probe <noop|eval7|eval7-formula|rs_poker|aya_poker|poker> [hold-ms] [touch-count]"
    );
    std::process::exit(2)
}
