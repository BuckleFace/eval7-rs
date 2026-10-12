use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use eval7_comparison_benchmarks::{
    aya_card, aya_poker_hand, eval7_card, eval7_hand, fixed_board_hands, native_hand, poker_card,
    poker_hand, rs_card, rs_poker_hand, sampled_card_ids,
};
use eval7_rs::benchmark_support::cards_are_distinct;
use rs_poker::core::{Rankable, SevenCardAccum};

const DATASET_SIZES: [usize; 3] = [64, 4_096, 262_144];
const POOL_SIZES: [usize; 4] = [1, 8, 64, 1_024];

fn cycle_benchmark<T: Copy + 'static>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    hands: &[T],
    mut evaluate: impl FnMut(T) + 'static,
) {
    let owned = hands.to_vec();
    let mut cursor = 0usize;
    group.bench_function(name, move |bencher| {
        bencher.iter(|| {
            let hand = black_box(owned[cursor]);
            cursor += 1;
            if cursor == owned.len() {
                cursor = 0;
            }
            evaluate(hand)
        });
    });
}

fn benchmark_random(criterion: &mut Criterion) {
    for count in DATASET_SIZES {
        let ids = sampled_card_ids(count);
        let eval7_hands: Vec<_> = ids.iter().copied().map(eval7_hand).collect();
        let rs_hands: Vec<_> = ids.iter().copied().map(rs_poker_hand).collect();
        let aya_hands: Vec<_> = ids.iter().copied().map(aya_poker_hand).collect();
        let poker_hands: Vec<_> = ids.iter().copied().map(poker_hand).collect();
        let mut group = criterion.benchmark_group(format!("random/{count}"));
        group.throughput(Throughput::Elements(1));

        let evaluator = eval7_rs::Evaluator::new();
        cycle_benchmark(&mut group, "eval7_checked", &eval7_hands, move |hand| {
            black_box(evaluator.evaluate_7(hand).unwrap());
        });
        let evaluator = eval7_rs::Evaluator::new();
        cycle_benchmark(&mut group, "eval7_unchecked", &eval7_hands, move |hand| {
            black_box(evaluator.evaluate_7_unchecked(hand));
        });
        cycle_benchmark(&mut group, "rs_poker", &rs_hands, |hand| {
            black_box(hand.rank());
        });
        cycle_benchmark(&mut group, "aya_poker", &aya_hands, |hand| {
            black_box(aya_poker::poker_rank(&hand));
        });
        let evaluator = poker::Evaluator::new();
        cycle_benchmark(&mut group, "poker", &poker_hands, move |hand| {
            black_box(evaluator.evaluate_five(hand).unwrap());
        });
        group.finish();
    }
}

fn benchmark_repeated(criterion: &mut Criterion) {
    let source = sampled_card_ids(*POOL_SIZES.last().unwrap());
    for count in POOL_SIZES {
        let ids = &source[..count];
        let eval7_hands: Vec<_> = ids.iter().copied().map(eval7_hand).collect();
        let rs_hands: Vec<_> = ids.iter().copied().map(rs_poker_hand).collect();
        let aya_hands: Vec<_> = ids.iter().copied().map(aya_poker_hand).collect();
        let poker_hands: Vec<_> = ids.iter().copied().map(poker_hand).collect();
        let mut group = criterion.benchmark_group(format!("repeated_pool/{count}"));
        let evaluator = eval7_rs::Evaluator::new();
        cycle_benchmark(&mut group, "eval7_checked", &eval7_hands, move |hand| {
            black_box(evaluator.evaluate_7(hand).unwrap());
        });
        let evaluator = eval7_rs::Evaluator::new();
        cycle_benchmark(&mut group, "eval7_unchecked", &eval7_hands, move |hand| {
            black_box(evaluator.evaluate_7_unchecked(hand));
        });
        cycle_benchmark(&mut group, "rs_poker", &rs_hands, |hand| {
            black_box(hand.rank());
        });
        cycle_benchmark(&mut group, "aya_poker", &aya_hands, |hand| {
            black_box(aya_poker::poker_rank(&hand));
        });
        let evaluator = poker::Evaluator::new();
        cycle_benchmark(&mut group, "poker", &poker_hands, move |hand| {
            black_box(evaluator.evaluate_five(hand).unwrap());
        });
        group.finish();
    }
}

fn benchmark_fixed_boards(criterion: &mut Criterion) {
    let boards = [
        ("dry", [51, 24, 31, 2, 39]),
        ("paired", [37, 24, 44, 2, 26]),
        ("monotone", [38, 35, 32, 28, 26]),
        ("connected", [46, 19, 31, 4, 39]),
    ];

    for (texture, board) in boards {
        let ids = fixed_board_hands(board);
        let eval7_hands: Vec<_> = ids.iter().copied().map(eval7_hand).collect();
        let rs_hands: Vec<_> = ids.iter().copied().map(rs_poker_hand).collect();
        let aya_hands: Vec<_> = ids.iter().copied().map(aya_poker_hand).collect();
        let poker_hands: Vec<_> = ids.iter().copied().map(poker_hand).collect();
        let mut group = criterion.benchmark_group(format!("fixed_board/{texture}"));
        let evaluator = eval7_rs::Evaluator::new();
        cycle_benchmark(&mut group, "eval7_checked", &eval7_hands, move |hand| {
            black_box(evaluator.evaluate_7(hand).unwrap());
        });
        let evaluator = eval7_rs::Evaluator::new();
        cycle_benchmark(&mut group, "eval7_unchecked", &eval7_hands, move |hand| {
            black_box(evaluator.evaluate_7_unchecked(hand));
        });
        cycle_benchmark(&mut group, "rs_poker", &rs_hands, |hand| {
            black_box(hand.rank());
        });
        cycle_benchmark(&mut group, "aya_poker", &aya_hands, |hand| {
            black_box(aya_poker::poker_rank(&hand));
        });
        let evaluator = poker::Evaluator::new();
        cycle_benchmark(&mut group, "poker", &poker_hands, move |hand| {
            black_box(evaluator.evaluate_five(hand).unwrap());
        });

        let board_cards = board.map(rs_card);
        let mut prefix = SevenCardAccum::new();
        for card in board_cards {
            prefix.add(card);
        }
        let holes: Vec<_> = ids
            .iter()
            .map(|hand| [rs_card(hand[5]), rs_card(hand[6])])
            .collect();
        let mut cursor = 0usize;
        group.bench_function("rs_poker_shared_prefix", |bencher| {
            bencher.iter(|| {
                let mut accumulated = black_box(prefix);
                let hole = black_box(holes[cursor]);
                cursor += 1;
                if cursor == holes.len() {
                    cursor = 0;
                }
                accumulated.add(hole[0]);
                accumulated.add(hole[1]);
                black_box(accumulated.rank())
            });
        });
        group.finish();
    }
}

fn benchmark_batches(criterion: &mut Criterion) {
    for count in [4_096, 262_144] {
        let ids = sampled_card_ids(count);
        let eval7_hands: Vec<_> = ids.iter().copied().map(eval7_hand).collect();
        let rs_hands: Vec<_> = ids.iter().copied().map(rs_poker_hand).collect();
        let aya_hands: Vec<_> = ids.iter().copied().map(aya_poker_hand).collect();
        let poker_hands: Vec<_> = ids.iter().copied().map(poker_hand).collect();
        let mut group = criterion.benchmark_group(format!("sequential_batch/{count}"));
        group.throughput(Throughput::Elements(count as u64));

        let evaluator = eval7_rs::Evaluator::new();
        group.bench_function("eval7_checked", |bencher| {
            bencher.iter(|| {
                for &hand in black_box(&eval7_hands) {
                    black_box(evaluator.evaluate_7(hand).unwrap());
                }
            });
        });
        let evaluator = eval7_rs::Evaluator::new();
        group.bench_function("eval7_unchecked", |bencher| {
            bencher.iter(|| {
                for &hand in black_box(&eval7_hands) {
                    black_box(evaluator.evaluate_7_unchecked(hand));
                }
            });
        });
        group.bench_function("rs_poker", |bencher| {
            bencher.iter(|| {
                for hand in black_box(&rs_hands) {
                    black_box(hand.rank());
                }
            });
        });
        group.bench_function("aya_poker", |bencher| {
            bencher.iter(|| {
                for hand in black_box(&aya_hands) {
                    black_box(aya_poker::poker_rank(hand));
                }
            });
        });
        let evaluator = poker::Evaluator::new();
        group.bench_function("poker", |bencher| {
            bencher.iter(|| {
                for &hand in black_box(&poker_hands) {
                    black_box(evaluator.evaluate_five(hand).unwrap());
                }
            });
        });
        group.finish();
    }
}

fn benchmark_api_overheads(criterion: &mut Criterion) {
    let ids = sampled_card_ids(4_096);
    let hand = eval7_hand(ids[0]);
    let evaluator = eval7_rs::Evaluator::new();
    let mut group = criterion.benchmark_group("api_overhead");

    group.bench_function("duplicate_validation", |bencher| {
        bencher.iter(|| black_box(cards_are_distinct(black_box(hand))))
    });
    group.bench_function("eval7_method_unchecked", |bencher| {
        bencher.iter(|| black_box(evaluator.evaluate_7_unchecked(black_box(hand))))
    });
    group.bench_function("eval7_free_function_unchecked", |bencher| {
        bencher.iter(|| black_box(eval7_rs::evaluate_7_unchecked(black_box(hand))))
    });

    let mut cursor = 0usize;
    group.bench_function("convert_one/eval7", |bencher| {
        bencher.iter(|| {
            let id = black_box(ids[cursor][0]);
            cursor = (cursor + 1) & (ids.len() - 1);
            black_box(eval7_card(id))
        })
    });
    let mut cursor = 0usize;
    group.bench_function("convert_one/rs_poker", |bencher| {
        bencher.iter(|| {
            let id = black_box(ids[cursor][0]);
            cursor = (cursor + 1) & (ids.len() - 1);
            black_box(rs_card(id))
        })
    });
    let mut cursor = 0usize;
    group.bench_function("convert_one/aya_poker", |bencher| {
        bencher.iter(|| {
            let id = black_box(ids[cursor][0]);
            cursor = (cursor + 1) & (ids.len() - 1);
            black_box(aya_card(id))
        })
    });
    let mut cursor = 0usize;
    group.bench_function("convert_one/poker", |bencher| {
        bencher.iter(|| {
            let id = black_box(ids[cursor][0]);
            cursor = (cursor + 1) & (ids.len() - 1);
            black_box(poker_card(id))
        })
    });
    let mut cursor = 0usize;
    group.bench_function("convert_full_native_set", |bencher| {
        bencher.iter(|| {
            let hand_ids = black_box(ids[cursor]);
            cursor = (cursor + 1) & (ids.len() - 1);
            black_box(native_hand(hand_ids))
        })
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .sample_size(80);
    targets =
        benchmark_random,
        benchmark_repeated,
        benchmark_fixed_boards,
        benchmark_batches,
        benchmark_api_overheads
}
criterion_main!(benches);
