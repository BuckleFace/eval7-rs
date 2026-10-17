//! Diagnostic hooks for repository benchmarks; not the ordinary evaluation path.
//! Legacy canonical-key helpers remain for compatibility. Their index values
//! now address the compact payload, not the retired V1 canonical table.

use crate::canonical::{
    canonical_key_7_by_suit_masks, canonical_key_from_sorted_suit_masks, sort_suit_masks,
    suit_masks_7,
};
use crate::evaluator::validate_distinct;
use crate::hybrid::HandState;
use crate::table::{ENTRY_COUNT, rank_at};
use crate::{Card, HandRank};

/// Embedded compact table bytes, including its 64-byte header.
pub const EMBEDDED_TABLE_BYTES: usize = 64 + ENTRY_COUNT * 2;
/// The compact backend has no heap-allocated formula indexer.
pub const FORMULA_INDEXER_HEAP_BYTES: usize = 0;
/// Inline bytes of the legacy-named, stateless benchmark handle.
pub const FORMULA_INDEXER_INLINE_BYTES: usize = size_of::<FreshFormulaIndexer>();
/// Number of directly addressable rank slots, including unused flush masks.
pub const EMBEDDED_TABLE_ENTRIES: usize = ENTRY_COUNT;

/// Runs only the production duplicate-card validation step.
#[inline]
pub fn cards_are_distinct(cards: [Card; 7]) -> bool {
    validate_distinct(cards).is_ok()
}

/// Builds four original 13-bit suit masks for diagnostic compatibility.
#[inline]
pub fn suit_masks(cards: [Card; 7]) -> [u16; 4] {
    suit_masks_7(cards)
}

/// Sorts suit masks for the legacy canonical-key diagnostic helpers.
#[inline]
pub fn sorted_suit_masks(masks: [u16; 4]) -> [u16; 4] {
    sort_suit_masks(masks)
}

/// Packs already-sorted suit masks into the legacy canonical representation.
#[inline]
pub fn canonical_key_from_sorted_masks(masks: [u16; 4]) -> u64 {
    canonical_key_from_sorted_suit_masks(masks)
}

/// Constructs a legacy suit-independent diagnostic key for seven distinct cards.
#[inline]
pub fn canonical_key(cards: [Card; 7]) -> u64 {
    canonical_key_7_by_suit_masks(cards)
}

/// Maps a valid seven-card canonical key to the compact payload's rank slot.
/// This compatibility helper includes reconstructing and aggregating a
/// suit-equivalent hand; it is not a standalone production index-stage timing.
#[inline]
pub fn formula_index(canonical_key: u64) -> usize {
    let mut state = HandState::empty();
    for suit in 0..4 {
        let mut mask = (canonical_key >> (13 * suit)) & 0x1fff;
        while mask != 0 {
            let rank = mask.trailing_zeros();
            state.add(Card::new((13 * suit + rank) as u8).expect("masked card ID is valid"));
            mask &= mask - 1;
        }
    }
    state.index()
}

/// Reads and decodes a rank from a compact payload index.
#[inline]
pub fn embedded_rank_at(index: usize) -> HandRank {
    rank_at(index)
}

/// Legacy-named initialization benchmark handle. Construction now allocates
/// nothing: the compact evaluator's indexing data is entirely static.
pub struct FreshFormulaIndexer {
    _private: (),
}

impl FreshFormulaIndexer {
    /// Creates a stateless compatibility handle without initializing tables.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Returns the number of directly addressable compact rank slots.
    pub fn total_count(&self) -> usize {
        ENTRY_COUNT
    }
}

impl Default for FreshFormulaIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Evaluator;

    #[test]
    fn exposed_stages_compose_to_the_production_result() {
        for ids in [
            [0, 14, 28, 42, 8, 22, 36],
            [0, 1, 2, 3, 4, 5, 6],
            [12, 25, 38, 51, 11, 23, 2],
        ] {
            let cards = ids.map(|id| Card::new(id).unwrap());
            let key = canonical_key_from_sorted_masks(sorted_suit_masks(suit_masks(cards)));
            assert_eq!(key, canonical_key(cards));
            let index = formula_index(key);
            assert!(index < EMBEDDED_TABLE_ENTRIES);
            assert_eq!(
                embedded_rank_at(index),
                Evaluator::new().evaluate_7_unchecked(cards)
            );
        }
        assert_eq!(
            FreshFormulaIndexer::new().total_count(),
            EMBEDDED_TABLE_ENTRIES
        );
        assert_eq!(FORMULA_INDEXER_HEAP_BYTES, 0);
    }
}
