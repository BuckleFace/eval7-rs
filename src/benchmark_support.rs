//! Internal hooks used only by this repository's benchmark targets.

use crate::evaluator::validate_distinct;
use crate::formula::Canonical7FormulaIndexer;
use crate::{Card, Evaluator, HandRank};

/// Embedded table bytes, including the 64-byte header.
pub const EMBEDDED_TABLE_BYTES: usize = 64 + 6_009_159 * 2;

/// Heap bytes requested by a formula indexer, excluding allocator metadata.
pub const FORMULA_INDEXER_HEAP_BYTES: usize = 8_192 + 5 * 8_193 * 8 * 4;

/// Inline bytes occupied by the formula indexer on the current target.
pub const FORMULA_INDEXER_INLINE_BYTES: usize = size_of::<Canonical7FormulaIndexer>();

/// Evaluator access that omits duplicate-card validation for known-distinct hands.
pub struct UncheckedEvaluator {
    evaluator: Evaluator,
}

impl UncheckedEvaluator {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
        }
    }

    #[inline]
    pub fn evaluate_distinct(&self, cards: [Card; 7]) -> HandRank {
        self.evaluator.evaluate_7_distinct(cards)
    }
}

impl Default for UncheckedEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs only the production duplicate-card validation step.
#[inline]
pub fn cards_are_distinct(cards: [Card; 7]) -> bool {
    validate_distinct(cards).is_ok()
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
