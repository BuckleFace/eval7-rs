use std::sync::OnceLock;

pub(crate) type Canonical7Key = u64;

const MASK_BITS: usize = 13;
const MASK_COUNT: usize = 1 << MASK_BITS;
const TOTAL_CARDS: usize = 7;
const TAIL_SLOTS: usize = 4;
const REM_COUNT: usize = TOTAL_CARDS + 1;
const TAIL_LO_COUNT: usize = MASK_COUNT + 1;

/// Ranks sorted four-suit-mask keys in a dense, lexicographic order.
///
/// A valid key is a nondecreasing tuple `(m0, m1, m2, m3)` whose mask
/// popcounts sum to seven. The index is its zero-based position among all such
/// tuples, ordered first by `m0`, then `m1`, and so on.
pub(crate) struct Canonical7FormulaIndexer {
    // `popcounts[m]` is the number of cards represented by suit mask `m`.
    popcounts: Box<[u8]>,
    // Flattened `T(slots, lo, remaining)` rows; see `new` for the recurrence.
    tails: Box<[[u32; REM_COUNT]]>,
    #[allow(dead_code)]
    total_count: usize,
}

impl Canonical7FormulaIndexer {
    pub(crate) fn new() -> Self {
        let popcounts = (0usize..MASK_COUNT)
            .map(|mask| mask.count_ones() as u8)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let mut tails =
            vec![[0u32; REM_COUNT]; (TAIL_SLOTS + 1) * TAIL_LO_COUNT].into_boxed_slice();

        // Let T(s, lo, r) count nondecreasing sequences of `s` masks, each at
        // least `lo`, whose popcounts sum to `r`. There is one empty sequence
        // with no cards, irrespective of the lower bound.
        for lo in 0..=MASK_COUNT {
            tails[tail_row_index(0, lo)][0] = 1;
        }

        // Partition the sequences counted by T(s, lo, r) according to whether
        // their first mask is greater than `lo` or exactly `lo`:
        //
        // T(s, lo, r) = T(s, lo + 1, r)
        //              + T(s - 1, lo, r - popcount(lo)).
        //
        // Keeping `lo` for the suffix permits equal adjacent masks. Iterating
        // lower bounds downward makes both terms available when a row is built.
        for slots in 1..=TAIL_SLOTS {
            for lo in (0..MASK_COUNT).rev() {
                let next_row = tails[tail_row_index(slots, lo + 1)];
                let prev_row = tails[tail_row_index(slots - 1, lo)];
                let row = tail_row_index(slots, lo);
                let popcount = popcounts[lo] as usize;

                for rem in 0..=TOTAL_CARDS {
                    let with_first = if popcount <= rem {
                        prev_row[rem - popcount]
                    } else {
                        0
                    };
                    tails[row][rem] = next_row[rem] + with_first;
                }
            }
        }

        let total_count = tails[tail_row_index(TAIL_SLOTS, 0)][TOTAL_CARDS] as usize;
        Self {
            popcounts,
            tails,
            total_count,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn total_count(&self) -> usize {
        self.total_count
    }

    #[inline(always)]
    pub(crate) fn index_for_key(&self, key: Canonical7Key) -> usize {
        let [m0, m1, m2, m3] = unpack_canonical_key(key);
        debug_assert!(m0 <= m1 && m1 <= m2 && m2 <= m3);

        let pc0 = self.popcounts[m0] as usize;
        let pc1 = self.popcounts[m1] as usize;
        let pc2 = self.popcounts[m2] as usize;
        let pc3 = self.popcounts[m3] as usize;
        debug_assert_eq!(pc0 + pc1 + pc2 + pc3, TOTAL_CARDS);

        // Count all valid tuples whose first differing mask is smaller than
        // the corresponding mask in this key. For a fixed prefix, subtracting
        // two tail counts selects suffixes whose next mask lies between the
        // prefix's lower bound (inclusive) and the current mask (exclusive).
        let mut index =
            self.tail(TAIL_SLOTS, 0, TOTAL_CARDS) - self.tail(TAIL_SLOTS, m0, TOTAL_CARDS);

        let rem_after_m0 = TOTAL_CARDS - pc0;
        index += self.tail(3, m0, rem_after_m0) - self.tail(3, m1, rem_after_m0);

        let rem_after_m1 = rem_after_m0 - pc1;
        index += self.tail(2, m1, rem_after_m1) - self.tail(2, m2, rem_after_m1);

        let rem_after_m2 = rem_after_m1 - pc2;
        debug_assert_eq!(pc3, rem_after_m2);
        index += self.tail(1, m2, rem_after_m2) - self.tail(1, m3, rem_after_m2);

        // These disjoint counts are precisely the lexicographic predecessors
        // of the key. Consequently every valid tuple has one rank, and those
        // ranks fill 0..total_count without gaps.
        index as usize
    }

    #[allow(dead_code)]
    pub(crate) fn try_index_for_key(&self, key: Canonical7Key) -> Option<usize> {
        let [m0, m1, m2, m3] = unpack_canonical_key(key);
        if !(m0 <= m1 && m1 <= m2 && m2 <= m3) {
            return None;
        }

        let total = self.popcounts[m0] as usize
            + self.popcounts[m1] as usize
            + self.popcounts[m2] as usize
            + self.popcounts[m3] as usize;
        if total != TOTAL_CARDS {
            return None;
        }
        Some(self.index_for_key(key))
    }

    #[inline(always)]
    fn tail(&self, slots: usize, lo: usize, rem: usize) -> u32 {
        self.tails[tail_row_index(slots, lo)][rem]
    }
}

impl Default for Canonical7FormulaIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub(crate) fn shared_indexer() -> &'static Canonical7FormulaIndexer {
    static INDEXER: OnceLock<Canonical7FormulaIndexer> = OnceLock::new();
    INDEXER.get_or_init(Canonical7FormulaIndexer::new)
}

#[inline(always)]
fn unpack_canonical_key(key: Canonical7Key) -> [usize; 4] {
    [
        ((key >> 39) & 0x1fff) as usize,
        ((key >> 26) & 0x1fff) as usize,
        ((key >> 13) & 0x1fff) as usize,
        (key & 0x1fff) as usize,
    ]
}

#[inline(always)]
fn tail_row_index(slots: usize, lo: usize) -> usize {
    debug_assert!(slots <= TAIL_SLOTS);
    debug_assert!(lo <= MASK_COUNT);
    slots * TAIL_LO_COUNT + lo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_indexer_has_expected_universe_size() {
        assert_eq!(Canonical7FormulaIndexer::new().total_count(), 6_009_159);
    }

    #[test]
    fn formula_indexer_maps_first_key_to_zero() {
        let indexer = Canonical7FormulaIndexer::new();
        assert_eq!(indexer.try_index_for_key(0b0000001111111), Some(0));
    }
}
