use std::cmp::Ordering;

use eval7_comparison_benchmarks::{
    Evaluators, category_aya_poker, category_eval7, category_poker, category_rs_poker, compare_all,
    native_hand, native_hands, sampled_card_ids,
};

const PREFLIGHT_HANDS: usize = 20_000;

fn main() {
    let evaluators = Evaluators::new();
    let hands = native_hands(&sampled_card_ids(PREFLIGHT_HANDS));

    for (index, hand) in hands.iter().enumerate() {
        let expected = category_eval7(eval7_comparison_benchmarks::eval7_rank(&evaluators, hand));
        let observed = [
            category_rs_poker(eval7_comparison_benchmarks::rs_poker_rank(hand)),
            category_aya_poker(eval7_comparison_benchmarks::aya_poker_rank(hand)),
            category_poker(eval7_comparison_benchmarks::poker_rank(&evaluators, hand)),
        ];
        assert!(
            observed.iter().all(|&category| category == expected),
            "category mismatch at deterministic fixture {index}: expected {expected:?}, got {observed:?}"
        );
    }

    for index in 0..hands.len() - 1 {
        let comparisons = compare_all(&evaluators, &hands[index], &hands[index + 1]);
        assert!(
            comparisons
                .iter()
                .all(|&ordering| ordering == comparisons[0]),
            "ordering mismatch for fixture pair {index}/{}: {comparisons:?}",
            index + 1
        );
    }

    let tie_left = native_hand([51, 24, 36, 9, 46, 14, 0]);
    let tie_right = native_hand([38, 50, 23, 35, 7, 15, 26]);
    let tie = compare_all(&evaluators, &tie_left, &tie_right);
    assert_eq!(tie, [Ordering::Equal; 4], "tie semantics differ: {tie:?}");

    println!(
        "correctness preflight passed: {PREFLIGHT_HANDS} categories, {} adjacent ordering comparisons, and one cross-suit tie",
        PREFLIGHT_HANDS - 1
    );
}
