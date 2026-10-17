//! Packed-card aggregation and flush/non-flush dispatch shared by ordinary and
//! prepared-board evaluation.

use crate::index::{NON_FLUSH_ENTRIES, non_flush_index};
use crate::table::rank_at;
use crate::{Card, HandRank};

const SUIT_BIAS: u64 = 0x3333u64 << 48;
const FLUSH_BITS: u64 = 0x8888u64 << 48;
const RANK_COUNT_MASK: u64 = (1u64 << 39) - 1;

const fn contributions() -> [[u64; 2]; 52] {
    let mut cards = [[0; 2]; 52];
    let mut id = 0;
    while id < 52 {
        cards[id] = [
            (1u64 << (3 * (id % 13))) | (1u64 << (48 + 4 * (id / 13))),
            1u64 << id,
        ];
        id += 1;
    }
    cards
}

static CONTRIBUTIONS: [[u64; 2]; 52] = contributions();

#[derive(Clone, Copy)]
pub(crate) struct HandState {
    // Thirteen three-bit rank counts, plus four biased suit-count nibbles.
    key: u64,
    // Original suit-major card bits: four adjacent 13-bit rank masks. Retain
    // suit alignment so a prepared board can be extended with real hole cards.
    cards: u64,
}

impl HandState {
    #[inline]
    pub(crate) fn empty() -> Self {
        Self {
            key: SUIT_BIAS,
            cards: 0,
        }
    }

    #[inline]
    pub(crate) fn from_cards(cards: [Card; 7]) -> Self {
        let mut state = Self::empty();
        for card in cards {
            state.add(card);
        }
        state
    }

    #[inline]
    pub(crate) fn contains(self, card: Card) -> bool {
        self.cards & (1u64 << card.id()) != 0
    }

    #[inline]
    pub(crate) fn add(&mut self, card: Card) {
        let [count, mask] = CONTRIBUTIONS[card.id() as usize];
        // At most seven additions: even duplicate unchecked inputs cannot
        // carry between three-bit rank digits or biased suit nibbles (max 10).
        self.key += count;
        self.cards |= mask;
    }

    /// Requires a seven-card aggregate; distinct cards are needed for its rank.
    #[inline]
    pub(crate) fn index(self) -> usize {
        // Bias three means a suit nibble's high bit becomes set at five cards.
        // With seven cards at most one suit qualifies; its bit is at 51+4*suit.
        let qualifying = self.key & FLUSH_BITS;
        if qualifying != 0 {
            let suit = (qualifying.trailing_zeros() - 51) / 4;
            let mask = ((self.cards >> (13 * suit)) & 0x1fff) as usize;
            // A flush leaves at most two off-suit cards. Quads and full houses
            // each require at least three, so neither can beat this flush.
            // The direct mask table already accounts for straight flushes.
            NON_FLUSH_ENTRIES + mask
        } else {
            non_flush_index(self.key & RANK_COUNT_MASK)
        }
    }

    #[inline]
    pub(crate) fn rank(self) -> HandRank {
        rank_at(self.index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_unchecked_cards_do_not_carry_between_fields() {
        for id in 0..52 {
            let state = HandState::from_cards([Card::new(id).unwrap(); 7]);
            assert_eq!(state.key & RANK_COUNT_MASK, 7 << (3 * (id % 13)));
            assert_eq!((state.key >> (48 + 4 * (id / 13))) & 15, 10);
        }
    }
}
