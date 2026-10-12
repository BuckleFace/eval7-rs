use crate::Card;

use crate::tables::{CARD_RANK_IDX, CARD_SUIT_IDX};

// Derived evaluator state for one 7-card showdown
// Packs the rank and suit aggregates that category evaluators read from
#[derive(Clone, Copy, Debug)]
pub(super) struct EvalState {
    // Rank multiplicities for deuce..ace mapped to 0..12
    pub(super) rank_counts: [u8; 13],
    // Suit multiplicities for suit ids 0..3
    pub(super) suit_counts: [u8; 4],
    // 13-bit mask of all ranks present across the 7 cards
    pub(super) rank_mask: u16,
    // Per-suit 13-bit rank masks
    pub(super) suit_rank_masks: [u16; 4],
}

// Derives aggregate evaluator state from seven showdown cards
#[inline]
pub(super) fn derive_state(cards: [Card; 7]) -> EvalState {
    let mut st = EvalState {
        rank_counts: [0; 13],
        suit_counts: [0; 4],
        rank_mask: 0,
        suit_rank_masks: [0; 4],
    };

    for card in cards {
        let id = card.id() as usize;
        debug_assert!(id < 52);

        let rank = CARD_RANK_IDX[id] as usize;
        let suit = CARD_SUIT_IDX[id] as usize;
        let bit = 1u16 << rank;

        st.rank_counts[rank] += 1;
        st.suit_counts[suit] += 1;
        st.rank_mask |= bit;
        st.suit_rank_masks[suit] |= bit;
    }

    debug_assert!(st.rank_counts.iter().map(|&x| x as u32).sum::<u32>() == 7);

    st
}
