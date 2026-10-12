use std::fmt;

use crate::canonical::canonical_key_7_by_suit_masks;
use crate::formula::{Canonical7FormulaIndexer, shared_indexer};
use crate::table::{ENTRY_COUNT, rank_at};
use crate::{Card, HandRank};

/// An error returned when seven cards do not form a valid distinct set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationError {
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

    /// Evaluates seven distinct cards.
    pub fn evaluate_7(&self, cards: [Card; 7]) -> Result<HandRank, EvaluationError> {
        validate_distinct(cards)?;
        let key = canonical_key_7_by_suit_masks(cards);
        let index = self.indexer.index_for_key(key);
        debug_assert!(index < ENTRY_COUNT);
        Ok(rank_at(index))
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluates seven distinct cards using the shared embedded evaluator.
pub fn evaluate_7(cards: [Card; 7]) -> Result<HandRank, EvaluationError> {
    Evaluator::new().evaluate_7(cards)
}

#[inline]
fn validate_distinct(cards: [Card; 7]) -> Result<(), EvaluationError> {
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
