use crate::Card;

pub(crate) type Canonical7Key = u64;

#[inline]
pub(crate) fn canonical_key_7_by_suit_masks(cards: [Card; 7]) -> Canonical7Key {
    let mut packed = 0u64;
    for card in cards {
        packed |= CARD_LANE_MASKS[card.id() as usize];
    }

    let mut masks = [
        (packed & MASK_13) as u16,
        ((packed >> 13) & MASK_13) as u16,
        ((packed >> 26) & MASK_13) as u16,
        ((packed >> 39) & MASK_13) as u16,
    ];
    masks.sort_unstable();

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
