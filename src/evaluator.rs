use std::fmt;

use crate::canonical::canonical_key_7_by_suit_masks;
use crate::formula::{Canonical7FormulaIndexer, shared_indexer};
use crate::table::{ENTRY_COUNT, rank_at};
use crate::{Card, HandRank};

/// An error returned when seven cards do not form a valid distinct set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationError {
    /// The same card occurs more than once in the seven-card input.
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

/// A reusable seven-card evaluator backed by the embedded rank table.
#[derive(Clone, Copy)]
pub struct Evaluator {
    indexer: &'static Canonical7FormulaIndexer,
}

impl Evaluator {
    /// Creates an evaluator, initializing the shared formula index on first use.
    pub fn new() -> Self {
        Self {
            indexer: shared_indexer(),
        }
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
        let key = canonical_key_7_by_suit_masks(cards);
        let index = self.indexer.index_for_key(key);
        debug_assert!(index < ENTRY_COUNT);
        rank_at(index)
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
