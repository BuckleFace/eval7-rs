use crate::canonical::canonical_key_from_packed_suits;
use crate::formula::{Canonical7FormulaIndexer, shared_indexer};
use crate::table::{ENTRY_COUNT, rank_at};
use crate::{Card, EvaluationError, HandRank};

/// A validated five-card board reusable with different two-card hands.
///
/// Uses the same evaluator and rank ordering as [`crate::evaluate_7`]. Reusing
/// the board avoids repeating its validation and aggregation, which can help
/// when several hands share a board; a speedup is not guaranteed.
///
/// The immutable object requires no per-board heap allocation. Like
/// [`crate::Evaluator::new`], construction initializes shared indexing data on first
/// use; that one-time initialization may allocate.
///
/// # Examples
///
/// ```
/// use eval7_rs::{Card, HandCategory, PreparedBoard};
///
/// let board = ["Ah", "Kh", "Qh", "2c", "3d"]
///     .map(|text| text.parse::<Card>().unwrap());
/// let prepared = PreparedBoard::new(board).unwrap();
/// let straight_flush = prepared.evaluate(["Jh", "Th"].map(|s| s.parse().unwrap())).unwrap();
/// let trips = prepared.evaluate(["As", "Ad"].map(|s| s.parse().unwrap())).unwrap();
///
/// assert_eq!(straight_flush.category(), HandCategory::StraightFlush);
/// assert_eq!(trips.category(), HandCategory::Trips);
/// assert!(straight_flush > trips);
/// ```
#[derive(Clone, Copy)]
pub struct PreparedBoard {
    // Preserve the original suit lanes until the hole cards are added:
    // sorting the board alone would lose their alignment with hole-card suits.
    board: u64,
    indexer: &'static Canonical7FormulaIndexer,
}

impl PreparedBoard {
    /// Prepares five distinct board cards in any order.
    ///
    /// Returns [`EvaluationError::DuplicateCard`] for the first repeated card.
    pub fn new(board: [Card; 5]) -> Result<Self, EvaluationError> {
        let board = extend_distinct(0, board)?;
        Ok(Self {
            board,
            indexer: shared_indexer(),
        })
    }

    /// Evaluates two hole cards against this board without changing it.
    ///
    /// Returns the rank of the best five-card hand among all seven cards. Neither
    /// hole card is required to be used.
    ///
    /// Returns [`EvaluationError::DuplicateCard`] if a hole card is already on
    /// the board or the two hole cards are identical. Hole-card order does not
    /// affect the rank. The same object can be reused after an error.
    #[inline]
    pub fn evaluate(&self, holes: [Card; 2]) -> Result<HandRank, EvaluationError> {
        let seven = extend_distinct(self.board, holes)?;
        // Validation establishes exactly seven bits before V1 indexing.
        let index = self
            .indexer
            .index_for_key(canonical_key_from_packed_suits(seven));
        debug_assert!(index < ENTRY_COUNT);
        Ok(rank_at(index))
    }
}

/// Validates and aggregates additional cards, including board overlap. Kept
/// local so ordinary evaluation retains its existing validation hot path.
#[inline]
fn extend_distinct<const N: usize>(
    mut seen: u64,
    cards: [Card; N],
) -> Result<u64, EvaluationError> {
    for card in cards {
        let bit = 1u64 << card.id();
        if seen & bit != 0 {
            return Err(EvaluationError::DuplicateCard(card));
        }
        seen |= bit;
    }
    Ok(seen)
}
