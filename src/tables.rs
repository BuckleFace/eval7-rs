// Card id -> rank and suit lookup tables
pub(super) const CARD_RANK_IDX: [u8; 52] = build_card_rank_idx();
pub(super) const CARD_SUIT_IDX: [u8; 52] = build_card_suit_idx();

// Builds the card id -> rank index table
// Rank indices use 0..12 for deuce..ace
const fn build_card_rank_idx() -> [u8; 52] {
    let mut table = [0u8; 52];
    let mut i = 0;

    while i < 52 {
        table[i] = (i % 13) as u8;
        i += 1;
    }

    table
}

// Builds the card id -> suit index table
// Suit indices use 0..3
const fn build_card_suit_idx() -> [u8; 52] {
    let mut table = [0u8; 52];
    let mut i = 0;

    while i < 52 {
        table[i] = (i / 13) as u8;
        i += 1;
    }

    table
}

// 13-bit rank mask -> straight high card lookup
// Returns 0 when no straight exists
pub(super) const STRAIGHT_HIGH_TABLE: [u8; 8192] = build_straight_high_table();

// Builds the rank-mask -> straight-high lookup table for all 13-bit masks
const fn build_straight_high_table() -> [u8; 8192] {
    let mut table = [0u8; 8192];
    let mut mask = 0usize;

    while mask < 8192 {
        table[mask] = straight_high_from_mask(mask as u16);
        mask += 1;
    }

    table
}

// Returns the straight high card for a 13-bit rank mask
// Uses rank values 2..14 and returns 5 for the wheel
const fn straight_high_from_mask(mask: u16) -> u8 {
    let mut high = 12usize;

    while high >= 4 {
        let need = (1u16 << high)
            | (1u16 << (high - 1))
            | (1u16 << (high - 2))
            | (1u16 << (high - 3))
            | (1u16 << (high - 4));

        if (mask & need) == need {
            return (high as u8) + 2;
        }

        if high == 4 {
            break;
        }

        high -= 1;
    }

    let wheel = (1u16 << 12) | (1u16 << 0) | (1u16 << 1) | (1u16 << 2) | (1u16 << 3);

    if (mask & wheel) == wheel { 5 } else { 0 }
}

// Binomial coefficients used for dense payload ranking
// CHOOSE[n][k] = n choose k
pub(super) const CHOOSE: [[u16; 6]; 14] = build_choose();

// Builds a small Pascal triangle used by payload encoders
const fn build_choose() -> [[u16; 6]; 14] {
    let mut table = [[0u16; 6]; 14];
    let mut n = 0usize;

    while n < 14 {
        table[n][0] = 1;
        n += 1;
    }

    n = 1;
    while n < 14 {
        let mut k = 1usize;
        while k <= 5 {
            table[n][k] = table[n - 1][k - 1] + table[n - 1][k];
            k += 1;
        }
        n += 1;
    }

    table
}
