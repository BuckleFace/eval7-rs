use std::panic::{AssertUnwindSafe, catch_unwind};

use eval7_rs::{
    Card, CardError, EvaluationError, Evaluator, HandCategory, PreparedBoard, evaluate_7,
    evaluate_7_unchecked,
};

fn cards<const N: usize>(text: [&str; N]) -> [Card; N] {
    text.map(|card| card.parse().expect("test card should be valid"))
}

#[test]
fn prepared_board_rejects_duplicates_and_remains_reusable() {
    let board = cards(["Ah", "Kh", "Qh", "2c", "3d"]);
    for a in 0..5 {
        for b in a + 1..5 {
            let mut duplicate = board;
            duplicate[b] = board[a];
            assert_eq!(
                PreparedBoard::new(duplicate).err(),
                Some(EvaluationError::DuplicateCard(board[a]))
            );
        }
    }
    let prepared = PreparedBoard::new(board).unwrap();
    for id in 0..52 {
        let card = Card::new(id).unwrap();
        assert_eq!(
            prepared.evaluate([card, card]),
            Err(EvaluationError::DuplicateCard(card))
        );
    }
    let valid = cards(["Jh", "Th"]);
    for card in board {
        for holes in [[card, valid[0]], [valid[0], card]] {
            assert_eq!(
                prepared.evaluate(holes),
                Err(EvaluationError::DuplicateCard(card))
            );
        }
    }
    for holes in [valid, cards(["As", "Ad"]), valid] {
        let seven = [
            board[0], board[1], board[2], board[3], board[4], holes[0], holes[1],
        ];
        assert_eq!(
            prepared.evaluate(holes).unwrap(),
            evaluate_7(seven).unwrap()
        );
        assert_eq!(
            prepared.evaluate([holes[1], holes[0]]),
            prepared.evaluate(holes)
        );
    }
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PreparedBoard>();
}

#[test]
fn prepared_board_handles_suit_permutations_and_board_partitions() {
    let cases = [
        cards(["Ac", "2d", "3h", "4s", "5c", "Kd", "Qh"]),
        cards(["Ac", "2c", "3c", "4c", "5c", "Kd", "Qh"]),
        cards(["Ac", "Ad", "Ah", "Ks", "Kc", "Kd", "2h"]),
        cards(["Ac", "Ad", "Ah", "As", "Kc", "4d", "2h"]),
        cards(["Ac", "Jc", "9c", "7c", "4c", "Kd", "2h"]),
    ];
    for hand in cases {
        let expected = evaluate_7(hand).unwrap();
        for a in 0..4 {
            for b in 0..4 {
                for c in 0..4 {
                    for d in 0..4 {
                        let suits = [a, b, c, d];
                        if (0..4).any(|i| suits[..i].contains(&suits[i])) {
                            continue;
                        }
                        let hand = hand.map(|card| {
                            Card::from_rank_suit(card.rank(), suits[card.suit() as usize]).unwrap()
                        });
                        for first in 0..7 {
                            for second in first + 1..7 {
                                let mut board = [hand[0]; 5];
                                let mut next = 0;
                                for (i, &card) in hand.iter().enumerate() {
                                    if i != first && i != second {
                                        board[next] = card;
                                        next += 1;
                                    }
                                }
                                let prepared = PreparedBoard::new(board).unwrap();
                                assert_eq!(
                                    prepared.evaluate([hand[first], hand[second]]).unwrap(),
                                    expected
                                );
                                board.reverse();
                                assert_eq!(
                                    PreparedBoard::new(board)
                                        .unwrap()
                                        .evaluate([hand[first], hand[second]])
                                        .unwrap(),
                                    expected
                                );
                            }
                        }
                    }
                }
            }
        }
    }
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
