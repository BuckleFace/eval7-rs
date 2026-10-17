//! Legacy canonical-key diagnostics for the optional benchmark hooks.
//! These helpers are not part of the compact evaluator's hot path.

use crate::Card;

/// Four 13-bit suit masks packed into a `u64` in ascending mask order.
pub(crate) type Canonical7Key = u64;

/// Returns a representation invariant under any permutation of the four suits.
///
/// Cards must be distinct for the masks to contain seven set bits in total.
#[inline]
pub(crate) fn canonical_key_7_by_suit_masks(cards: [Card; 7]) -> Canonical7Key {
    canonical_key_from_sorted_suit_masks(sort_suit_masks(suit_masks_7(cards)))
}

#[inline]
pub(crate) fn suit_masks_7(cards: [Card; 7]) -> [u16; 4] {
    let mut packed = 0u64;
    for card in cards {
        packed |= CARD_LANE_MASKS[card.id() as usize];
    }
    suit_masks_from_packed(packed)
}

#[inline]
fn suit_masks_from_packed(packed: u64) -> [u16; 4] {
    // Card IDs are suit-major. `card / 13` selects one of four adjacent
    // 13-bit lanes, while `card % 13` selects deuce through ace within it.
    [
        (packed & MASK_13) as u16,
        ((packed >> 13) & MASK_13) as u16,
        ((packed >> 26) & MASK_13) as u16,
        ((packed >> 39) & MASK_13) as u16,
    ]
}

#[inline]
pub(crate) fn sort_suit_masks(mut masks: [u16; 4]) -> [u16; 4] {
    // Renaming suits only permutes these masks, so their sorted sequence is a
    // unique representative of every suit-equivalent hand.
    masks.sort_unstable();
    masks
}

/// Packs four masks that are already sorted in ascending order.
///
/// Placing the smallest mask in the most significant lane makes numeric key
/// order agree with the lexicographic tuple order used by the retired V1 indexer.
#[inline]
pub(crate) fn canonical_key_from_sorted_suit_masks(masks: [u16; 4]) -> Canonical7Key {
    ((masks[0] as u64) << 39)
        | ((masks[1] as u64) << 26)
        | ((masks[2] as u64) << 13)
        | masks[3] as u64
}

const MASK_13: u64 = 0x1fff;
const CARD_LANE_MASKS: [u64; 52] = build_card_lane_masks();

const fn build_card_lane_masks() -> [u64; 52] {
    let mut masks = [0u64; 52];
    let mut card = 0;
    while card < 52 {
        masks[card] = 1u64 << ((card / 13) * 13 + card % 13);
        card += 1;
    }
    masks
}
