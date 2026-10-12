use std::panic::{AssertUnwindSafe, catch_unwind};

use eval7_rs::{
    Card, CardError, EvaluationError, Evaluator, HandCategory, evaluate_7, evaluate_7_unchecked,
};

fn cards(text: [&str; 7]) -> [Card; 7] {
    text.map(|card| card.parse().expect("test card should be valid"))
}

#[test]
fn public_api_evaluates_and_compares_hands() {
    let pair = evaluate_7(cards(["Ac", "Ad", "Kh", "9s", "7c", "4d", "2h"])).unwrap();
    let flush = Evaluator::new()
        .evaluate_7(cards(["Ac", "Jc", "9c", "7c", "4c", "Kd", "2h"]))
        .unwrap();

    assert_eq!(pair.category(), HandCategory::Pair);
    assert_eq!(flush.category(), HandCategory::Flush);
    assert!(flush > pair);
}

#[test]
fn public_api_rejects_duplicate_cards() {
    let duplicate = "Ac".parse::<Card>().unwrap();
    let result = evaluate_7([
        duplicate,
        "Ad".parse().unwrap(),
        "Kh".parse().unwrap(),
        "9s".parse().unwrap(),
        "7c".parse().unwrap(),
        "4d".parse().unwrap(),
        duplicate,
    ]);
    assert_eq!(result, Err(EvaluationError::DuplicateCard(duplicate)));
}

#[test]
fn unchecked_public_api_matches_checked_api() {
    let evaluator = Evaluator::new();
    let hand = cards(["Ac", "Ad", "Kh", "9s", "7c", "4d", "2h"]);
    let expected = evaluate_7(hand).unwrap();
    assert_eq!(evaluate_7_unchecked(hand), expected);
    assert_eq!(evaluator.evaluate_7(hand).unwrap(), expected);
    assert_eq!(evaluator.evaluate_7_unchecked(hand), expected);
}

#[test]
fn unchecked_duplicate_inputs_return_or_panic_safely() {
    let ace = "Ac".parse::<Card>().unwrap();
    let duplicate_cases = [
        [ace; 7],
        cards(["Ac", "Ac", "Kh", "9s", "7c", "4d", "2h"]),
        cards(["Ac", "Ac", "Kh", "Kh", "7c", "7c", "2h"]),
    ];

    for hand in duplicate_cases {
        if let Ok(rank) = catch_unwind(AssertUnwindSafe(|| evaluate_7_unchecked(hand))) {
            // Any returned value came from the bounded embedded table and must
            // retain a valid packed category. A panic is also documented behavior.
            let _ = rank.category();
        }
    }
}

#[test]
fn public_card_constructors_cannot_create_invalid_cards() {
    for id in 0..52 {
        assert!(Card::new(id).is_ok());
    }
    for id in 52..=u8::MAX {
        assert_eq!(Card::new(id), Err(CardError::InvalidId));
    }

    for rank in 0..=u8::MAX {
        for suit in 0..=u8::MAX {
            let valid = (2..=14).contains(&rank) && suit <= 3;
            assert_eq!(Card::from_rank_suit(rank, suit).is_ok(), valid);
        }
    }
}
