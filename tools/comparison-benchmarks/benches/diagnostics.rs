use std::hint::black_box;
use std::time::Duration;

use aya_poker::base as aya;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use eval7_comparison_benchmarks::{
    aya_card, aya_poker_hand, aya_poker_hand_checked, eval7_hand, fixed_board_hands, poker_card,
    poker_hand, rs_card, rs_poker_accumulator, rs_poker_hand, sampled_card_ids,
};
use eval7_rs::benchmark_support::{
    EMBEDDED_TABLE_ENTRIES, canonical_key, canonical_key_from_sorted_masks, embedded_rank_at,
    formula_index, sorted_suit_masks, suit_masks,
};
use eval7_rs::{Card as Eval7Card, Evaluator as Eval7Evaluator};
use poker::{Card as PokerCard, Evaluator as PokerEvaluator};
use rs_poker::core::{Hand as RsHand, Rankable, SevenCardAccum};

const WORKING_SET_SIZES: [usize; 3] = [64, 4_096, 262_144];
const FIXED_BOARDS: [(&str, [u8; 5]); 4] = [
    ("dry", [51, 24, 31, 2, 39]),
    ("paired", [37, 24, 44, 2, 26]),
    ("monotone", [38, 35, 32, 28, 26]),
    ("connected", [46, 19, 31, 4, 39]),
];

fn cycle_benchmark<T: Copy + 'static, R>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    inputs: &[T],
    mut operation: impl FnMut(T) -> R + 'static,
) {
    let owned = inputs.to_vec();
    let mut cursor = 0usize;
    group.bench_function(name, move |bencher| {
        bencher.iter(|| {
            let input = black_box(owned[cursor]);
            cursor += 1;
            if cursor == owned.len() {
                cursor = 0;
            }
            black_box(operation(input))
        });
    });
}

fn benchmark_raw_end_to_end(criterion: &mut Criterion) {
    for count in WORKING_SET_SIZES {
        let ids = sampled_card_ids(count);
        let mut group = criterion.benchmark_group(format!("raw_end_to_end/{count}"));
        group.throughput(Throughput::Elements(1));

        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(&mut group, "eval7_checked", &ids, move |ids| {
            evaluator.evaluate_7(eval7_hand(ids)).unwrap()
        });
        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(&mut group, "eval7_prevalidated", &ids, move |ids| {
            evaluator.evaluate_7_unchecked(eval7_hand(ids))
        });
        cycle_benchmark(&mut group, "aya_poker_checked", &ids, |ids| {
            aya_poker::poker_rank(&aya_poker_hand_checked(ids))
        });
        cycle_benchmark(&mut group, "aya_poker_prevalidated", &ids, |ids| {
            aya_poker::poker_rank(&aya_poker_hand(ids))
        });
        cycle_benchmark(&mut group, "rs_poker_hand_checked", &ids, |ids| {
            rs_poker_hand(ids).rank()
        });
        cycle_benchmark(
            &mut group,
            "rs_poker_accumulator_prevalidated",
            &ids,
            |ids| rs_poker_accumulator(ids).rank(),
        );
        let evaluator = PokerEvaluator::new();
        cycle_benchmark(&mut group, "poker_checked", &ids, move |ids| {
            evaluator.evaluate_five(poker_hand(ids)).unwrap()
        });
        group.finish();
    }
}

fn benchmark_native_construction(criterion: &mut Criterion) {
    for count in WORKING_SET_SIZES {
        let ids = sampled_card_ids(count);
        let mut group = criterion.benchmark_group(format!("native_construction/{count}"));
        group.throughput(Throughput::Elements(1));

        cycle_benchmark(&mut group, "eval7", &ids, eval7_hand);
        cycle_benchmark(
            &mut group,
            "aya_poker_checked",
            &ids,
            aya_poker_hand_checked,
        );
        cycle_benchmark(&mut group, "aya_poker_prevalidated", &ids, aya_poker_hand);
        cycle_benchmark(&mut group, "rs_poker_hand_checked", &ids, rs_poker_hand);
        cycle_benchmark(
            &mut group,
            "rs_poker_accumulator_prevalidated",
            &ids,
            rs_poker_accumulator,
        );
        cycle_benchmark(&mut group, "poker", &ids, poker_hand);
        group.finish();
    }
}

fn benchmark_preconstructed_evaluation(criterion: &mut Criterion) {
    for count in WORKING_SET_SIZES {
        let ids = sampled_card_ids(count);
        let eval7_hands: Vec<_> = ids.iter().copied().map(eval7_hand).collect();
        let aya_hands: Vec<_> = ids.iter().copied().map(aya_poker_hand).collect();
        let rs_hands: Vec<_> = ids.iter().copied().map(rs_poker_hand).collect();
        let rs_accumulators: Vec<_> = ids.iter().copied().map(rs_poker_accumulator).collect();
        let poker_hands: Vec<_> = ids.iter().copied().map(poker_hand).collect();
        let mut group = criterion.benchmark_group(format!("preconstructed_evaluation/{count}"));
        group.throughput(Throughput::Elements(1));

        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(&mut group, "eval7_checked", &eval7_hands, move |hand| {
            evaluator.evaluate_7(hand).unwrap()
        });
        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(
            &mut group,
            "eval7_prevalidated",
            &eval7_hands,
            move |hand| evaluator.evaluate_7_unchecked(hand),
        );
        cycle_benchmark(&mut group, "aya_poker", &aya_hands, |hand| {
            aya_poker::poker_rank(&hand)
        });
        cycle_benchmark(&mut group, "rs_poker_hand", &rs_hands, |hand| hand.rank());
        cycle_benchmark(
            &mut group,
            "rs_poker_accumulator",
            &rs_accumulators,
            |accumulator| accumulator.rank(),
        );
        let evaluator = PokerEvaluator::new();
        cycle_benchmark(&mut group, "poker", &poker_hands, move |hand| {
            evaluator.evaluate_five(hand).unwrap()
        });
        group.finish();
    }
}

fn benchmark_fixed_board_full_rebuild(criterion: &mut Criterion) {
    for (texture, board) in FIXED_BOARDS {
        let ids = fixed_board_hands(board);
        let mut group = criterion.benchmark_group(format!("fixed_board_raw_full/{texture}"));
        group.throughput(Throughput::Elements(1));

        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(&mut group, "eval7_checked", &ids, move |ids| {
            evaluator.evaluate_7(eval7_hand(ids)).unwrap()
        });
        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(&mut group, "eval7_prevalidated", &ids, move |ids| {
            evaluator.evaluate_7_unchecked(eval7_hand(ids))
        });
        cycle_benchmark(&mut group, "aya_poker_checked", &ids, |ids| {
            aya_poker::poker_rank(&aya_poker_hand_checked(ids))
        });
        cycle_benchmark(&mut group, "aya_poker_prevalidated", &ids, |ids| {
            aya_poker::poker_rank(&aya_poker_hand(ids))
        });
        cycle_benchmark(&mut group, "rs_poker_hand_checked", &ids, |ids| {
            rs_poker_hand(ids).rank()
        });
        cycle_benchmark(
            &mut group,
            "rs_poker_accumulator_prevalidated",
            &ids,
            |ids| rs_poker_accumulator(ids).rank(),
        );
        let evaluator = PokerEvaluator::new();
        cycle_benchmark(&mut group, "poker_checked", &ids, move |ids| {
            evaluator.evaluate_five(poker_hand(ids)).unwrap()
        });
        group.finish();
    }
}

fn eval7_with_board(board: [Eval7Card; 5], holes: [u8; 2]) -> [Eval7Card; 7] {
    [
        board[0],
        board[1],
        board[2],
        board[3],
        board[4],
        Eval7Card::new(holes[0]).unwrap(),
        Eval7Card::new(holes[1]).unwrap(),
    ]
}

fn poker_with_board(board: [PokerCard; 5], holes: [u8; 2]) -> [PokerCard; 7] {
    [
        board[0],
        board[1],
        board[2],
        board[3],
        board[4],
        poker_card(holes[0]),
        poker_card(holes[1]),
    ]
}

fn benchmark_fixed_board_incremental(criterion: &mut Criterion) {
    for (texture, board_ids) in FIXED_BOARDS {
        let hands = fixed_board_hands(board_ids);
        let holes: Vec<_> = hands.iter().map(|hand| [hand[5], hand[6]]).collect();
        let eval7_board = board_ids.map(|id| Eval7Card::new(id).unwrap());
        let poker_board = board_ids.map(poker_card);

        let mut aya_board = aya::Hand::new();
        for id in board_ids {
            aya_board.insert_unchecked(&aya_card(id));
        }
        let mut rs_board_hand = RsHand::new();
        let mut rs_board_accumulator = SevenCardAccum::new();
        for id in board_ids {
            let card = rs_card(id);
            assert!(rs_board_hand.insert(card));
            rs_board_accumulator.add(card);
        }

        let mut group = criterion.benchmark_group(format!("fixed_board_raw_incremental/{texture}"));
        group.throughput(Throughput::Elements(1));

        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(&mut group, "eval7_checked", &holes, move |holes| {
            evaluator
                .evaluate_7(eval7_with_board(eval7_board, holes))
                .unwrap()
        });
        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(&mut group, "eval7_prevalidated", &holes, move |holes| {
            evaluator.evaluate_7_unchecked(eval7_with_board(eval7_board, holes))
        });
        cycle_benchmark(&mut group, "aya_poker_checked", &holes, move |holes| {
            let mut hand = aya_board;
            hand.extend(holes.map(aya_card));
            aya_poker::poker_rank(&hand)
        });
        cycle_benchmark(&mut group, "aya_poker_prevalidated", &holes, move |holes| {
            let mut hand = aya_board;
            for id in holes {
                hand.insert_unchecked(&aya_card(id));
            }
            aya_poker::poker_rank(&hand)
        });
        cycle_benchmark(&mut group, "rs_poker_hand_checked", &holes, move |holes| {
            let mut hand = rs_board_hand;
            assert!(hand.insert(rs_card(holes[0])));
            assert!(hand.insert(rs_card(holes[1])));
            hand.rank()
        });
        cycle_benchmark(
            &mut group,
            "rs_poker_accumulator_prevalidated",
            &holes,
            move |holes| {
                let mut accumulator = rs_board_accumulator;
                accumulator.add(rs_card(holes[0]));
                accumulator.add(rs_card(holes[1]));
                accumulator.rank()
            },
        );
        let evaluator = PokerEvaluator::new();
        cycle_benchmark(&mut group, "poker_checked", &holes, move |holes| {
            evaluator
                .evaluate_five(poker_with_board(poker_board, holes))
                .unwrap()
        });
        group.finish();
    }
}

fn benchmark_eval7_stages(criterion: &mut Criterion) {
    for count in WORKING_SET_SIZES {
        let ids = sampled_card_ids(count);
        let cards: Vec<_> = ids.iter().copied().map(eval7_hand).collect();
        let masks: Vec<_> = cards.iter().copied().map(suit_masks).collect();
        let sorted_masks: Vec<_> = masks.iter().copied().map(sorted_suit_masks).collect();
        let keys: Vec<_> = cards.iter().copied().map(canonical_key).collect();
        let indices: Vec<_> = keys.iter().copied().map(formula_index).collect();
        let mut group = criterion.benchmark_group(format!("eval7_internal/{count}"));
        group.throughput(Throughput::Elements(1));

        cycle_benchmark(&mut group, "suit_masks", &cards, suit_masks);
        cycle_benchmark(&mut group, "sort_four_masks", &masks, sorted_suit_masks);
        cycle_benchmark(
            &mut group,
            "pack_key_from_sorted_masks",
            &sorted_masks,
            canonical_key_from_sorted_masks,
        );
        cycle_benchmark(&mut group, "canonical_key_complete", &cards, canonical_key);
        cycle_benchmark(&mut group, "formula_index", &keys, formula_index);
        cycle_benchmark(
            &mut group,
            "rank_table_scattered",
            &indices,
            embedded_rank_at,
        );
        let evaluator = Eval7Evaluator::new();
        cycle_benchmark(&mut group, "unchecked_complete", &cards, move |hand| {
            evaluator.evaluate_7_unchecked(hand)
        });
        group.finish();
    }
}

fn benchmark_eval7_table_locality(criterion: &mut Criterion) {
    for count in WORKING_SET_SIZES {
        let ids = sampled_card_ids(count);
        let scattered: Vec<_> = ids
            .iter()
            .copied()
            .map(eval7_hand)
            .map(canonical_key)
            .map(formula_index)
            .collect();
        let sequential: Vec<_> = (0..count)
            .map(|index| index % EMBEDDED_TABLE_ENTRIES)
            .collect();
        let mut group = criterion.benchmark_group(format!("eval7_table_access/{count}"));
        group.throughput(Throughput::Elements(1));
        cycle_benchmark(&mut group, "sequential", &sequential, embedded_rank_at);
        cycle_benchmark(&mut group, "scattered", &scattered, embedded_rank_at);
        group.finish();
    }
}

criterion_group! {
    name = diagnostics;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .sample_size(80);
    targets =
        benchmark_raw_end_to_end,
        benchmark_native_construction,
        benchmark_preconstructed_evaluation,
        benchmark_fixed_board_full_rebuild,
        benchmark_fixed_board_incremental,
        benchmark_eval7_stages,
        benchmark_eval7_table_locality
}
criterion_main!(diagnostics);
