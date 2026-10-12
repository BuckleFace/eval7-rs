use crate::direct::evaluate_7_direct;
use crate::reference::reference_eval_7;
use crate::{Card, Evaluator, HandCategory, evaluate_7};

fn cards(text: [&str; 7]) -> [Card; 7] {
    text.map(|card| card.parse().expect("test card should be valid"))
}

#[test]
fn detects_every_hand_category() {
    let cases = [
        (
            ["Ac", "Kd", "9h", "7s", "4c", "3d", "2h"],
            HandCategory::HighCard,
        ),
        (
            ["Ac", "Ad", "Kh", "9s", "7c", "4d", "2h"],
            HandCategory::Pair,
        ),
        (
            ["Ac", "Ad", "Kh", "Ks", "7c", "4d", "2h"],
            HandCategory::TwoPair,
        ),
        (
            ["Ac", "Ad", "Ah", "Ks", "7c", "4d", "2h"],
            HandCategory::Trips,
        ),
        (
            ["Ac", "Kd", "Qh", "Js", "Tc", "4d", "2h"],
            HandCategory::Straight,
        ),
        (
            ["Ac", "Jc", "9c", "7c", "4c", "Kd", "2h"],
            HandCategory::Flush,
        ),
        (
            ["Ac", "Ad", "Ah", "Ks", "Kc", "4d", "2h"],
            HandCategory::FullHouse,
        ),
        (
            ["Ac", "Ad", "Ah", "As", "Kc", "4d", "2h"],
            HandCategory::Quads,
        ),
        (
            ["Ac", "Kc", "Qc", "Jc", "Tc", "4d", "2h"],
            HandCategory::StraightFlush,
        ),
    ];

    for (text, expected) in cases {
        let rank = evaluate_7(cards(text)).unwrap();
        assert_eq!(rank.category(), expected, "cards: {text:?}");
    }
}

#[test]
fn category_and_pair_ordering_is_preserved() {
    let category_hands = [
        ["Ac", "Kd", "9h", "7s", "4c", "3d", "2h"],
        ["Ac", "Ad", "Kh", "9s", "7c", "4d", "2h"],
        ["Ac", "Ad", "Kh", "Ks", "7c", "4d", "2h"],
        ["Ac", "Ad", "Ah", "Ks", "7c", "4d", "2h"],
        ["Ac", "Kd", "Qh", "Js", "Tc", "4d", "2h"],
        ["Ac", "Jc", "9c", "7c", "4c", "Kd", "2h"],
        ["Ac", "Ad", "Ah", "Ks", "Kc", "4d", "2h"],
        ["Ac", "Ad", "Ah", "As", "Kc", "4d", "2h"],
        ["Ac", "Kc", "Qc", "Jc", "Tc", "4d", "2h"],
    ];
    let ranks = category_hands.map(|hand| evaluate_7(cards(hand)).unwrap());
    assert!(ranks.windows(2).all(|pair| pair[0] < pair[1]));

    let kings = evaluate_7(cards(["Kc", "Kd", "Ah", "Qs", "9c", "4d", "2h"])).unwrap();
    let aces = evaluate_7(cards(["Ac", "Ad", "Kh", "Qs", "9c", "4d", "2h"])).unwrap();
    assert!(aces > kings);

    let same_pair_lower_kicker =
        evaluate_7(cards(["Ac", "Ad", "Kh", "Js", "8c", "4d", "2h"])).unwrap();
    let same_pair_higher_kicker =
        evaluate_7(cards(["Ac", "Ad", "Kh", "Qs", "8c", "4d", "2h"])).unwrap();
    assert!(same_pair_higher_kicker > same_pair_lower_kicker);
}

#[test]
fn embedded_table_matches_original_direct_algorithm_on_representative_hands() {
    let evaluator = Evaluator::new();
    let hands = [
        ["Ac", "Ad", "Kh", "9s", "7c", "4d", "2h"],
        ["Ac", "Kd", "Qh", "Js", "Tc", "4d", "2h"],
        ["Ac", "Jc", "9c", "7c", "4c", "Kd", "2h"],
        ["Ac", "Ad", "Ah", "Ks", "Kc", "4d", "2h"],
        ["5c", "4c", "3c", "2c", "Ac", "Kd", "Qh"],
    ];

    for hand in hands.map(cards) {
        assert_eq!(evaluator.evaluate_7(hand).unwrap(), evaluate_7_direct(hand));
        assert_eq!(
            evaluator.evaluate_7(hand).unwrap(),
            reference_eval_7(hand).rank
        );
    }
}

#[test]
fn deterministic_sample_matches_original_direct_and_reference_evaluators() {
    const SAMPLE_COUNT: usize = 50_000;
    let evaluator = Evaluator::new();
    let mut seed = 0xc0ffee1234abcdefu64;

    for sample in 0..SAMPLE_COUNT {
        let mut deck: [Card; 52] =
            std::array::from_fn(|id| Card::new(id as u8).expect("deck id should be valid"));
        for i in 0..7 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let j = i + seed as usize % (52 - i);
            deck.swap(i, j);
        }
        let hand: [Card; 7] = deck[..7].try_into().unwrap();
        let embedded = evaluator.evaluate_7(hand).unwrap();
        let direct = evaluate_7_direct(hand);
        let reference = reference_eval_7(hand);
        assert_eq!(
            embedded, direct,
            "embedded/direct mismatch at sample {sample}"
        );
        assert_eq!(
            direct, reference.rank,
            "direct/reference mismatch at sample {sample}; best five: {:?}",
            reference.best_five
        );
    }
}
