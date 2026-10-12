use eval7_rs::{Card, EvaluationError, Evaluator, HandCategory, evaluate_7};

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
