use crate::hybrid::HandState;
use crate::{Card, EvaluationError, HandRank};

/// A validated five-card board reusable with different two-card hands.
///
/// Uses the same evaluator and rank ordering as [`crate::evaluate_7`]. Reusing
/// the board avoids repeating its validation and aggregation, which can help
/// when several hands share a board; a speedup is not guaranteed.
///
/// The immutable object requires no heap allocation or shared initialization;
/// all indexing support and rank tables are embedded in the library.
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
    state: HandState,
}

impl PreparedBoard {
    /// Prepares five distinct board cards in any order.
    ///
    /// Returns [`EvaluationError::DuplicateCard`] for the first repeated card.
    pub fn new(board: [Card; 5]) -> Result<Self, EvaluationError> {
        Ok(Self {
            state: extend_distinct(HandState::empty(), board)?,
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
        // Copying the aggregate leaves this board unchanged, including after
        // validation errors. Adding two cards gives the ordinary seven-card
        // aggregate: count addition and suit-mask union are associative.
        Ok(extend_distinct(self.state, holes)?.rank())
    }
}

/// Validates and aggregates additional cards, including board overlap. Kept
/// local so ordinary evaluation retains its existing validation hot path.
#[inline]
fn extend_distinct<const N: usize>(
    mut state: HandState,
    cards: [Card; N],
) -> Result<HandState, EvaluationError> {
    for card in cards {
        if state.contains(card) {
            return Err(EvaluationError::DuplicateCard(card));
        }
        state.add(card);
    }
    Ok(state)
}
