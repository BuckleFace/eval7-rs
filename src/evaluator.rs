use std::fmt;

use crate::hybrid::HandState;
use crate::{Card, HandRank};

/// An error returned when evaluation input contains a repeated card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationError {
    /// A card repeats within the input or overlaps a prepared board.
    DuplicateCard(Card),
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCard(card) => write!(f, "duplicate card {card}"),
        }
    }
}

impl std::error::Error for EvaluationError {}

/// A stateless seven-card evaluator backed by embedded compact rank tables.
#[derive(Clone, Copy)]
pub struct Evaluator {
    _private: (),
}

impl Evaluator {
    /// Creates an evaluator without allocation or runtime table initialization.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Evaluates seven cards, rejecting a duplicate with
    /// [`EvaluationError::DuplicateCard`].
    pub fn evaluate_7(&self, cards: [Card; 7]) -> Result<HandRank, EvaluationError> {
        validate_distinct(cards)?;
        Ok(self.evaluate_7_unchecked(cards))
    }

    /// Evaluates seven cards without checking that they are distinct.
    ///
    /// The caller is responsible for supplying a valid seven-card combination.
    /// Duplicate cards may produce a meaningless result or panic, but cannot
    /// cause undefined behavior. This path is intended for performance-sensitive
    /// callers that have already established card uniqueness.
    #[inline]
    pub fn evaluate_7_unchecked(&self, cards: [Card; 7]) -> HandRank {
        HandState::from_cards(cards).rank()
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluates seven cards using the shared embedded evaluator.
///
/// Returns [`EvaluationError::DuplicateCard`] if a card occurs more than once.
///
/// # Examples
///
/// ```
/// use eval7_rs::{Card, HandCategory, evaluate_7};
///
/// let cards = ["Ah", "Kh", "Qh", "Jh", "Th", "2c", "3d"]
///     .map(|text| text.parse::<Card>().unwrap());
/// let rank = evaluate_7(cards).unwrap();
///
/// assert_eq!(rank.category(), HandCategory::StraightFlush);
/// ```
pub fn evaluate_7(cards: [Card; 7]) -> Result<HandRank, EvaluationError> {
    Evaluator::new().evaluate_7(cards)
}

/// Evaluates seven cards without checking that they are distinct.
///
/// The caller is responsible for supplying a valid seven-card combination.
/// Duplicate cards may produce a meaningless result or panic, but cannot cause
/// undefined behavior. This safe function skips application-level validation;
/// it does not perform any unchecked memory access.
///
/// # Examples
///
/// ```
/// use eval7_rs::{Card, HandCategory, evaluate_7_unchecked};
///
/// // This input has already been established to contain seven distinct cards.
/// let cards = ["As", "Ad", "Ac", "Ah", "Kd", "Qc", "2s"]
///     .map(|text| text.parse::<Card>().unwrap());
/// let rank = evaluate_7_unchecked(cards);
///
/// assert_eq!(rank.category(), HandCategory::Quads);
/// ```
pub fn evaluate_7_unchecked(cards: [Card; 7]) -> HandRank {
    Evaluator::new().evaluate_7_unchecked(cards)
}

#[inline]
pub(crate) fn validate_distinct(cards: [Card; 7]) -> Result<(), EvaluationError> {
    let mut seen = 0u64;
    for card in cards {
        let bit = 1u64 << card.id();
        if seen & bit != 0 {
            return Err(EvaluationError::DuplicateCard(card));
        }
        seen |= bit;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_cards_are_rejected() {
        let cards = [
            Card::new(0).unwrap(),
            Card::new(1).unwrap(),
            Card::new(2).unwrap(),
            Card::new(3).unwrap(),
            Card::new(4).unwrap(),
            Card::new(5).unwrap(),
            Card::new(0).unwrap(),
        ];
        assert_eq!(
            evaluate_7(cards),
            Err(EvaluationError::DuplicateCard(Card::new(0).unwrap()))
        );
    }
}
