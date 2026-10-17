use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use eval7_rs::benchmark_support::cards_are_distinct;
use eval7_rs::{Card, Evaluator, PreparedBoard};

const RANDOM_HAND_COUNT: usize = 4_096;

fn cards(text: [&str; 7]) -> [Card; 7] {
    text.map(|card| card.parse().expect("benchmark card should be valid"))
}

fn random_hands() -> Vec<[Card; 7]> {
    let mut seed = 0xc0ff_ee12_34ab_cdefu64;
    let mut hands = Vec::with_capacity(RANDOM_HAND_COUNT);

    for _ in 0..RANDOM_HAND_COUNT {
        let mut hand = [Card::new(0).unwrap(); 7];
        let mut seen = 0u64;
        for card in &mut hand {
            loop {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let id = ((seed >> 32) % 52) as u8;
                let bit = 1u64 << id;
                if seen & bit == 0 {
                    seen |= bit;
                    *card = Card::new(id).unwrap();
                    break;
                }
            }
        }
        hands.push(hand);
    }
    hands
}

fn fixed_board_hands() -> Vec<[Card; 7]> {
    let board = ["As", "Kd", "7h", "4c", "2s"].map(|card| card.parse::<Card>().unwrap());
    let board_mask = board
        .iter()
        .fold(0u64, |mask, card| mask | (1u64 << card.id()));
    let available: Vec<Card> = (0..52)
        .map(|id| Card::new(id).unwrap())
        .filter(|card| board_mask & (1u64 << card.id()) == 0)
        .collect();
    let mut hands = Vec::with_capacity(1_081);
    for first in 0..available.len() - 1 {
        for second in first + 1..available.len() {
            hands.push([
                board[0],
                board[1],
                board[2],
                board[3],
                board[4],
                available[first],
                available[second],
            ]);
        }
    }
    assert_eq!(hands.len(), 1_081);
    hands
}

fn benchmark_repeated_hand(criterion: &mut Criterion) {
    let evaluator = Evaluator::new();
    let hand = cards(["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]);
    let mut group = criterion.benchmark_group("repeated_hand");

    group.bench_function("public_checked", |bencher| {
        bencher.iter(|| evaluator.evaluate_7(black_box(hand)).unwrap())
    });
    group.bench_function("public_unchecked", |bencher| {
        bencher.iter(|| evaluator.evaluate_7_unchecked(black_box(hand)))
    });
    group.bench_function("duplicate_validation_only", |bencher| {
        bencher.iter(|| cards_are_distinct(black_box(hand)))
    });
    group.finish();
}

fn benchmark_distributed_hands(criterion: &mut Criterion) {
    let evaluator = Evaluator::new();
    let hands = random_hands();
    let mut group = criterion.benchmark_group("random_distributed");

    let mut checked_cursor = 0usize;
    group.bench_function("public_checked", |bencher| {
        bencher.iter(|| {
            let hand = hands[checked_cursor];
            checked_cursor = (checked_cursor + 1) & (RANDOM_HAND_COUNT - 1);
            evaluator.evaluate_7(black_box(hand)).unwrap()
        })
    });

    let mut unchecked_cursor = 0usize;
    group.bench_function("public_unchecked", |bencher| {
        bencher.iter(|| {
            let hand = hands[unchecked_cursor];
            unchecked_cursor = (unchecked_cursor + 1) & (RANDOM_HAND_COUNT - 1);
            evaluator.evaluate_7_unchecked(black_box(hand))
        })
    });
    group.finish();
}

fn benchmark_fixed_board(criterion: &mut Criterion) {
    let evaluator = Evaluator::new();
    let hands = fixed_board_hands();
    let prepared = PreparedBoard::new(hands[0][..5].try_into().unwrap()).unwrap();
    let mut group = criterion.benchmark_group("fixed_board");

    let mut checked_cursor = 0usize;
    group.bench_function("public_checked", |bencher| {
        bencher.iter(|| {
            let hand = hands[checked_cursor];
            checked_cursor += 1;
            if checked_cursor == hands.len() {
                checked_cursor = 0;
            }
            evaluator.evaluate_7(black_box(hand)).unwrap()
        })
    });

    let mut unchecked_cursor = 0usize;
    group.bench_function("public_unchecked", |bencher| {
        bencher.iter(|| {
            let hand = hands[unchecked_cursor];
            unchecked_cursor += 1;
            if unchecked_cursor == hands.len() {
                unchecked_cursor = 0;
            }
            evaluator.evaluate_7_unchecked(black_box(hand))
        })
    });
    let mut prepared_cursor = 0usize;
    group.bench_function("prepared_checked", |bencher| {
        bencher.iter(|| {
            let hand = black_box(hands[prepared_cursor]);
            prepared_cursor += 1;
            if prepared_cursor == hands.len() {
                prepared_cursor = 0;
            }
            prepared.evaluate([hand[5], hand[6]]).unwrap()
        })
    });
    group.finish();
}

fn benchmark_sequential_batches(criterion: &mut Criterion) {
    let evaluator = Evaluator::new();
    let hands = random_hands();
    let mut group = criterion.benchmark_group("sequential_batch");
    group.throughput(Throughput::Elements(hands.len() as u64));

    group.bench_with_input(
        BenchmarkId::new("public_checked", hands.len()),
        &hands,
        |bencher, hands| {
            bencher.iter(|| {
                let mut checksum = 0u64;
                for &hand in hands {
                    checksum = checksum
                        .wrapping_add(evaluator.evaluate_7(black_box(hand)).unwrap().raw() as u64);
                }
                black_box(checksum)
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("public_unchecked", hands.len()),
        &hands,
        |bencher, hands| {
            bencher.iter(|| {
                let mut checksum = 0u64;
                for &hand in hands {
                    checksum = checksum
                        .wrapping_add(evaluator.evaluate_7_unchecked(black_box(hand)).raw() as u64);
                }
                black_box(checksum)
            })
        },
    );
    group.finish();
}

fn benchmark_initialization(criterion: &mut Criterion) {
    criterion.bench_function("initialization/evaluator_construct", |bencher| {
        bencher.iter(|| black_box(Evaluator::new()))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .sample_size(100);
    targets =
        benchmark_repeated_hand,
        benchmark_distributed_hands,
        benchmark_fixed_board,
        benchmark_sequential_batches,
        benchmark_initialization
}
criterion_main!(benches);
