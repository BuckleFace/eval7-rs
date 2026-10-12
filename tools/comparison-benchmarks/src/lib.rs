use std::cmp::Ordering;

use aya_poker::{PokerHandRank, PokerRankCategory, base as aya};
use eval7_rs::{Card as Eval7Card, Evaluator as Eval7Evaluator, HandCategory, HandRank};
use poker::{Card as PokerCard, Evaluator as PokerEvaluator, Rank as PokerRank, Suit as PokerSuit};
use rs_poker::core::{
    Card as RsCard, CoreRank, Hand as RsHand, Rank as RsRank, Rankable, Suit as RsSuit,
    Value as RsValue,
};

pub const COMBINATION_COUNT: u64 = 133_784_560;
pub const SAMPLE_STRIDE: u64 = 1_000_003;
pub const SAMPLE_OFFSET: u64 = 42_424_243;

pub type CardIds = [u8; 7];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    HighCard,
    Pair,
    TwoPair,
    Trips,
    Straight,
    Flush,
    FullHouse,
    Quads,
    StraightFlush,
}

#[derive(Clone, Copy)]
pub struct NativeHand {
    pub eval7: [Eval7Card; 7],
    pub rs_poker: RsHand,
    pub aya_poker: aya::Hand,
    pub poker: [PokerCard; 7],
}

pub struct Evaluators {
    pub eval7: Eval7Evaluator,
    pub poker: PokerEvaluator,
}

impl Evaluators {
    pub fn new() -> Self {
        Self {
            eval7: Eval7Evaluator::new(),
            poker: PokerEvaluator::new(),
        }
    }
}

impl Default for Evaluators {
    fn default() -> Self {
        Self::new()
    }
}

pub fn choose(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result = 1u64;
    for divisor in 1..=k {
        result = result * (n - k + divisor) / divisor;
    }
    result
}

/// Returns the lexicographically `rank`th seven-card combination.
pub fn unrank_combination(mut rank: u64) -> CardIds {
    assert!(rank < COMBINATION_COUNT);
    let mut cards = [0u8; 7];
    let mut next = 0u8;

    for (position, card) in cards.iter_mut().enumerate() {
        let remaining = 6 - position;
        loop {
            let block = choose(51 - u64::from(next), remaining as u64);
            if rank < block {
                *card = next;
                next += 1;
                break;
            }
            rank -= block;
            next += 1;
        }
    }
    cards
}

/// Deterministic sampling without replacement over all C(52, 7) combinations.
pub fn sampled_card_ids(count: usize) -> Vec<CardIds> {
    assert!(count as u64 <= COMBINATION_COUNT);
    (0..count).map(sampled_card_id).collect()
}

pub fn sampled_card_id(index: usize) -> CardIds {
    assert!((index as u64) < COMBINATION_COUNT);
    let rank = (SAMPLE_OFFSET + (index as u64) * SAMPLE_STRIDE) % COMBINATION_COUNT;
    unrank_combination(rank)
}

pub fn fixed_board_hands(board: [u8; 5]) -> Vec<CardIds> {
    let board_mask = board.iter().fold(0u64, |mask, &card| mask | (1u64 << card));
    let available: Vec<u8> = (0..52)
        .filter(|card| board_mask & (1u64 << card) == 0)
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

pub fn native_hand(ids: CardIds) -> NativeHand {
    NativeHand {
        eval7: eval7_hand(ids),
        rs_poker: rs_poker_hand(ids),
        aya_poker: aya_poker_hand(ids),
        poker: poker_hand(ids),
    }
}

pub fn eval7_hand(ids: CardIds) -> [Eval7Card; 7] {
    ids.map(|id| Eval7Card::new(id).expect("fixture card id is valid"))
}

pub fn rs_poker_hand(ids: CardIds) -> RsHand {
    let mut rs_poker = RsHand::new();
    for id in ids {
        assert!(rs_poker.insert(rs_card(id)));
    }
    rs_poker
}

pub fn aya_poker_hand(ids: CardIds) -> aya::Hand {
    let mut aya_poker = aya::Hand::new();
    for id in ids {
        aya_poker.insert_unchecked(&aya_card(id));
    }
    aya_poker
}

pub fn poker_hand(ids: CardIds) -> [PokerCard; 7] {
    ids.map(poker_card)
}

pub fn native_hands(ids: &[CardIds]) -> Vec<NativeHand> {
    ids.iter().copied().map(native_hand).collect()
}

pub fn eval7_rank(evaluators: &Evaluators, hand: &NativeHand) -> HandRank {
    evaluators.eval7.evaluate_7_unchecked(hand.eval7)
}

pub fn rs_poker_rank(hand: &NativeHand) -> RsRank {
    hand.rs_poker.rank()
}

pub fn aya_poker_rank(hand: &NativeHand) -> PokerHandRank {
    aya_poker::poker_rank(&hand.aya_poker)
}

pub fn poker_rank(evaluators: &Evaluators, hand: &NativeHand) -> poker::Eval<poker::FiveCard> {
    evaluators
        .poker
        .evaluate_five(hand.poker)
        .expect("fixture is a valid seven-card hand")
}

pub fn category_eval7(rank: HandRank) -> Category {
    match rank.category() {
        HandCategory::HighCard => Category::HighCard,
        HandCategory::Pair => Category::Pair,
        HandCategory::TwoPair => Category::TwoPair,
        HandCategory::Trips => Category::Trips,
        HandCategory::Straight => Category::Straight,
        HandCategory::Flush => Category::Flush,
        HandCategory::FullHouse => Category::FullHouse,
        HandCategory::Quads => Category::Quads,
        HandCategory::StraightFlush => Category::StraightFlush,
    }
}

pub fn category_rs_poker(rank: RsRank) -> Category {
    match rank.category() {
        CoreRank::HighCard => Category::HighCard,
        CoreRank::OnePair => Category::Pair,
        CoreRank::TwoPair => Category::TwoPair,
        CoreRank::ThreeOfAKind => Category::Trips,
        CoreRank::Straight => Category::Straight,
        CoreRank::Flush => Category::Flush,
        CoreRank::FullHouse => Category::FullHouse,
        CoreRank::FourOfAKind => Category::Quads,
        CoreRank::StraightFlush => Category::StraightFlush,
    }
}

pub fn category_aya_poker(rank: PokerHandRank) -> Category {
    match rank.rank_category() {
        PokerRankCategory::HighCard => Category::HighCard,
        PokerRankCategory::Pair => Category::Pair,
        PokerRankCategory::TwoPair => Category::TwoPair,
        PokerRankCategory::ThreeOfAKind => Category::Trips,
        PokerRankCategory::Straight => Category::Straight,
        PokerRankCategory::Flush => Category::Flush,
        PokerRankCategory::FullHouse => Category::FullHouse,
        PokerRankCategory::FourOfAKind => Category::Quads,
        PokerRankCategory::StraightFlush | PokerRankCategory::RoyalFlush => Category::StraightFlush,
        PokerRankCategory::Ineligible => panic!("seven-card fixture was ineligible"),
    }
}

pub fn category_poker(rank: poker::Eval<poker::FiveCard>) -> Category {
    use poker::evaluate::FiveCardHandClass;
    match rank.classify() {
        FiveCardHandClass::HighCard { .. } => Category::HighCard,
        FiveCardHandClass::Pair { .. } => Category::Pair,
        FiveCardHandClass::TwoPair { .. } => Category::TwoPair,
        FiveCardHandClass::ThreeOfAKind { .. } => Category::Trips,
        FiveCardHandClass::Straight { .. } => Category::Straight,
        FiveCardHandClass::Flush { .. } => Category::Flush,
        FiveCardHandClass::FullHouse { .. } => Category::FullHouse,
        FiveCardHandClass::FourOfAKind { .. } => Category::Quads,
        FiveCardHandClass::StraightFlush { .. } => Category::StraightFlush,
    }
}

pub fn compare_all(
    evaluators: &Evaluators,
    left: &NativeHand,
    right: &NativeHand,
) -> [Ordering; 4] {
    [
        eval7_rank(evaluators, left).cmp(&eval7_rank(evaluators, right)),
        rs_poker_rank(left).cmp(&rs_poker_rank(right)),
        aya_poker_rank(left).cmp(&aya_poker_rank(right)),
        poker_rank(evaluators, left).cmp(&poker_rank(evaluators, right)),
    ]
}

pub fn eval7_card(id: u8) -> Eval7Card {
    Eval7Card::new(id).expect("card id must be below 52")
}

pub fn rs_card(id: u8) -> RsCard {
    const SUITS: [RsSuit; 4] = [RsSuit::Club, RsSuit::Diamond, RsSuit::Heart, RsSuit::Spade];
    RsCard::new(RsValue::from_u8(id % 13), SUITS[(id / 13) as usize])
}

pub fn aya_card(id: u8) -> aya::Card {
    let rank = aya::Rank::try_from(id % 13).expect("card id must be below 52");
    let suit = aya::Suit::try_from(id / 13).expect("card id must be below 52");
    aya::Card::new(rank, suit)
}

pub fn poker_card(id: u8) -> PokerCard {
    const RANKS: [PokerRank; 13] = [
        PokerRank::Two,
        PokerRank::Three,
        PokerRank::Four,
        PokerRank::Five,
        PokerRank::Six,
        PokerRank::Seven,
        PokerRank::Eight,
        PokerRank::Nine,
        PokerRank::Ten,
        PokerRank::Jack,
        PokerRank::Queen,
        PokerRank::King,
        PokerRank::Ace,
    ];
    const SUITS: [PokerSuit; 4] = [
        PokerSuit::Clubs,
        PokerSuit::Diamonds,
        PokerSuit::Hearts,
        PokerSuit::Spades,
    ];
    PokerCard::new(RANKS[(id % 13) as usize], SUITS[(id / 13) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combination_unranking_has_expected_edges() {
        assert_eq!(choose(52, 7), COMBINATION_COUNT);
        assert_eq!(unrank_combination(0), [0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(
            unrank_combination(COMBINATION_COUNT - 1),
            [45, 46, 47, 48, 49, 50, 51]
        );
    }

    #[test]
    fn sample_stride_covers_without_repetition() {
        fn gcd(mut left: u64, mut right: u64) -> u64 {
            while right != 0 {
                (left, right) = (right, left % right);
            }
            left
        }
        assert_eq!(gcd(SAMPLE_STRIDE, COMBINATION_COUNT), 1);
        let hands = sampled_card_ids(10_000);
        let unique: std::collections::HashSet<_> = hands.iter().collect();
        assert_eq!(unique.len(), hands.len());
        assert!(
            hands
                .iter()
                .all(|hand| hand.windows(2).all(|w| w[0] < w[1]))
        );
    }
}
