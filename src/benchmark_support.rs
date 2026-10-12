//! Internal hooks used only by this repository's benchmark targets.

use crate::Card;
use crate::HandRank;
use crate::canonical::{
    canonical_key_7_by_suit_masks, canonical_key_from_sorted_suit_masks, sort_suit_masks,
    suit_masks_7,
};
use crate::evaluator::validate_distinct;
use crate::formula::{Canonical7FormulaIndexer, shared_indexer};
use crate::table::{ENTRY_COUNT, rank_at};

/// Embedded table bytes, including the 64-byte header.
pub const EMBEDDED_TABLE_BYTES: usize = 64 + 6_009_159 * 2;

/// Heap bytes requested by a formula indexer, excluding allocator metadata.
pub const FORMULA_INDEXER_HEAP_BYTES: usize = 8_192 + 5 * 8_193 * 8 * 4;

/// Inline bytes occupied by the formula indexer on the current target.
pub const FORMULA_INDEXER_INLINE_BYTES: usize = size_of::<Canonical7FormulaIndexer>();

/// Number of ranks in the embedded dense table.
pub const EMBEDDED_TABLE_ENTRIES: usize = ENTRY_COUNT;

/// Runs only the production duplicate-card validation step.
#[inline]
pub fn cards_are_distinct(cards: [Card; 7]) -> bool {
    validate_distinct(cards).is_ok()
}

/// Builds the four unsorted 13-bit suit masks used by production canonicalisation.
#[inline]
pub fn suit_masks(cards: [Card; 7]) -> [u16; 4] {
    suit_masks_7(cards)
}

/// Sorts four suit masks using the production canonicalisation operation.
#[inline]
pub fn sorted_suit_masks(masks: [u16; 4]) -> [u16; 4] {
    sort_suit_masks(masks)
}

/// Packs four already-sorted suit masks into a canonical key.
#[inline]
pub fn canonical_key_from_sorted_masks(masks: [u16; 4]) -> u64 {
    canonical_key_from_sorted_suit_masks(masks)
}

/// Runs complete production suit-mask canonicalisation.
#[inline]
pub fn canonical_key(cards: [Card; 7]) -> u64 {
    canonical_key_7_by_suit_masks(cards)
}

/// Calculates a dense formula index from an already-constructed canonical key.
#[inline]
pub fn formula_index(canonical_key: u64) -> usize {
    shared_indexer().index_for_key(canonical_key)
}

/// Reads and decodes a rank from an already-calculated dense table index.
#[inline]
pub fn embedded_rank_at(index: usize) -> HandRank {
    rank_at(index)
}

/// A freshly allocated formula indexer for initialization benchmarks.
pub struct FreshFormulaIndexer {
    indexer: Canonical7FormulaIndexer,
}

impl FreshFormulaIndexer {
    pub fn new() -> Self {
        Self {
            indexer: Canonical7FormulaIndexer::new(),
        }
    }

    pub fn total_count(&self) -> usize {
        self.indexer.total_count()
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
        let cards = [0, 14, 28, 42, 8, 22, 36].map(|id| Card::new(id).unwrap());
        let masks = suit_masks(cards);
        let sorted = sorted_suit_masks(masks);
        let key = canonical_key_from_sorted_masks(sorted);
        assert_eq!(key, canonical_key(cards));

        let index = formula_index(key);
        assert!(index < EMBEDDED_TABLE_ENTRIES);
        assert_eq!(
            embedded_rank_at(index),
            Evaluator::new().evaluate_7_unchecked(cards)
        );
    }
}
