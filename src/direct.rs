use crate::Card;

use crate::rank::{
    CAT_FLUSH, CAT_FULL_HOUSE, CAT_HIGH_CARD, CAT_PAIR, CAT_QUADS, CAT_STRAIGHT,
    CAT_STRAIGHT_FLUSH, CAT_TRIPS, CAT_TWO_PAIR, HandRank, pack,
};
use crate::state::{EvalState, derive_state};
use crate::tables::{CHOOSE, STRAIGHT_HIGH_TABLE};

pub(crate) fn evaluate_7_direct(cards: [Card; 7]) -> HandRank {
    let st = derive_state(cards);

    if let Some(rank) = eval_straight_flush(&st) {
        return rank;
    }
    if let Some(rank) = eval_quads(&st) {
        return rank;
    }
    if let Some(rank) = eval_full_house(&st) {
        return rank;
    }
    if let Some(rank) = eval_flush(&st) {
        return rank;
    }
    if let Some(rank) = eval_straight(&st) {
        return rank;
    }
    if let Some(rank) = eval_trips(&st) {
        return rank;
    }
    if let Some(rank) = eval_two_pair(&st) {
        return rank;
    }
    if let Some(rank) = eval_pair(&st) {
        return rank;
    }
    eval_high_card(&st)
}

// --- helper functions for rank evaluation ---

// Returns a mask containing the top `n` set ranks from a rank mask
#[inline]
fn top_n_mask(mask: u16, n: usize) -> u16 {
    let mut out = 0u16;
    let mut taken = 0usize;

    for rank in (0..13).rev() {
        let bit = 1u16 << rank;
        if (mask & bit) != 0 {
            out |= bit;
            taken += 1;
            if taken == n {
                break;
            }
        }
    }

    out
}

// Removes one excluded rank from ordinal space for payload encoding
#[inline]
fn compress1(rank: usize, dead: usize) -> usize {
    if rank < dead { rank } else { rank - 1 }
}

// Removes two excluded ranks from ordinal space for payload encoding
#[inline]
fn compress2(rank: usize, a: usize, b: usize) -> usize {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };

    if rank < lo {
        rank
    } else if rank < hi {
        rank - 1
    } else {
        rank - 2
    }
}

// --- payload encoding functions ---

// Encodes a 5-rank mask into its category-local payload ordinal
#[inline]
fn rank5_payload_from_mask(mask: u16) -> u16 {
    debug_assert!(mask.count_ones() == 5);

    let mut ranks = [0usize; 5];
    let mut n = 0usize;

    for rank in 0..13 {
        if (mask & (1u16 << rank)) != 0 {
            ranks[n] = rank;
            n += 1;
        }
    }

    debug_assert!(n == 5);

    CHOOSE[ranks[0]][1]
        + CHOOSE[ranks[1]][2]
        + CHOOSE[ranks[2]][3]
        + CHOOSE[ranks[3]][4]
        + CHOOSE[ranks[4]][5]
}

// Encodes pair rank plus three kickers into a pair payload
#[inline]
fn pair_payload(pair_rank: usize, k0: usize, k1: usize, k2: usize) -> u16 {
    let a = compress1(k0, pair_rank);
    let b = compress1(k1, pair_rank);
    let c = compress1(k2, pair_rank);

    let kicker_ordinal = CHOOSE[a][1] + CHOOSE[b][2] + CHOOSE[c][3];

    (pair_rank as u16) * 220 + kicker_ordinal
}

// Encodes two pair plus kicker into a two-pair payload
#[inline]
fn two_pair_payload(high_pair: usize, low_pair: usize, kicker: usize) -> u16 {
    debug_assert!(low_pair < high_pair);

    let pair_block = CHOOSE[low_pair][1] + CHOOSE[high_pair][2];

    let kicker_ordinal = compress2(kicker, low_pair, high_pair) as u16;

    pair_block * 11 + kicker_ordinal
}

// Encodes trips plus two kickers into a trips payload
#[inline]
fn trips_payload(trips_rank: usize, k0: usize, k1: usize) -> u16 {
    let a = compress1(k0, trips_rank);
    let b = compress1(k1, trips_rank);

    let kicker_ordinal = CHOOSE[a][1] + CHOOSE[b][2];

    (trips_rank as u16) * 66 + kicker_ordinal
}

// Encodes quads plus kicker into a quads payload
#[inline]
fn quads_payload(quad_rank: usize, kicker: usize) -> u16 {
    let kicker_ordinal = compress1(kicker, quad_rank) as u16;
    (quad_rank as u16) * 12 + kicker_ordinal
}

// Encodes trips rank plus pair rank into a full-house payload
#[inline]
fn full_house_payload(trips_rank: usize, pair_rank: usize) -> u16 {
    let pair_ordinal = compress1(pair_rank, trips_rank) as u16;
    (trips_rank as u16) * 12 + pair_ordinal
}

// --- evaluation functions for each hand category ---

// Evaluates straight flushes by suit
#[inline]
fn eval_straight_flush(st: &EvalState) -> Option<HandRank> {
    for suit in 0..4 {
        if st.suit_counts[suit] >= 5 {
            let high = STRAIGHT_HIGH_TABLE[st.suit_rank_masks[suit] as usize];
            if high != 0 {
                let payload = (high as u16) - 5;
                return Some(pack(CAT_STRAIGHT_FLUSH, payload));
            }
        }
    }

    None
}

// Evaluates four of a kind and kicker
#[inline]
fn eval_quads(st: &EvalState) -> Option<HandRank> {
    for quad_rank in (0..13).rev() {
        if st.rank_counts[quad_rank] == 4 {
            for kicker in (0..13).rev() {
                if kicker != quad_rank && st.rank_counts[kicker] != 0 {
                    let payload = quads_payload(quad_rank, kicker);
                    return Some(pack(CAT_QUADS, payload));
                }
            }

            debug_assert!(false, "quad hand must have a kicker");
        }
    }

    None
}

// Evaluates the best full house if present
#[inline]
fn eval_full_house(st: &EvalState) -> Option<HandRank> {
    for trips_rank in (0..13).rev() {
        if st.rank_counts[trips_rank] >= 3 {
            for pair_rank in (0..13).rev() {
                if pair_rank != trips_rank && st.rank_counts[pair_rank] >= 2 {
                    let payload = full_house_payload(trips_rank, pair_rank);
                    return Some(pack(CAT_FULL_HOUSE, payload));
                }
            }
        }
    }

    None
}

// Evaluates a flush using the top five ranks of the best qualifying suit
#[inline]
fn eval_flush(st: &EvalState) -> Option<HandRank> {
    for suit in 0..4 {
        if st.suit_counts[suit] >= 5 {
            let top5 = top_n_mask(st.suit_rank_masks[suit], 5);
            let payload = rank5_payload_from_mask(top5);
            return Some(pack(CAT_FLUSH, payload));
        }
    }

    None
}

// Evaluates a straight from the global rank mask
#[inline]
fn eval_straight(st: &EvalState) -> Option<HandRank> {
    let high = STRAIGHT_HIGH_TABLE[st.rank_mask as usize];

    if high != 0 {
        let payload = (high as u16) - 5;
        Some(pack(CAT_STRAIGHT, payload))
    } else {
        None
    }
}

// Evaluates trips plus the two best kickers
#[inline]
fn eval_trips(st: &EvalState) -> Option<HandRank> {
    for trips_rank in (0..13).rev() {
        if st.rank_counts[trips_rank] >= 3 {
            let mut kickers_desc = [0usize; 2];
            let mut n = 0usize;

            for rank in (0..13).rev() {
                if rank != trips_rank && st.rank_counts[rank] != 0 {
                    kickers_desc[n] = rank;
                    n += 1;
                    if n == 2 {
                        break;
                    }
                }
            }

            debug_assert!(n == 2);

            let payload = trips_payload(trips_rank, kickers_desc[1], kickers_desc[0]);
            return Some(pack(CAT_TRIPS, payload));
        }
    }

    None
}

// Evaluates the best two pair plus kicker
#[inline]
fn eval_two_pair(st: &EvalState) -> Option<HandRank> {
    let mut pairs_desc = [0usize; 2];
    let mut n = 0usize;

    for rank in (0..13).rev() {
        if st.rank_counts[rank] >= 2 {
            pairs_desc[n] = rank;
            n += 1;
            if n == 2 {
                break;
            }
        }
    }

    if n < 2 {
        return None;
    }

    let high_pair = pairs_desc[0];
    let low_pair = pairs_desc[1];

    for kicker in (0..13).rev() {
        if kicker != high_pair && kicker != low_pair && st.rank_counts[kicker] != 0 {
            let payload = two_pair_payload(high_pair, low_pair, kicker);
            return Some(pack(CAT_TWO_PAIR, payload));
        }
    }

    debug_assert!(false, "two-pair hand must have a kicker");
    None
}

// Evaluates one pair plus the three best kickers
#[inline]
fn eval_pair(st: &EvalState) -> Option<HandRank> {
    for pair_rank in (0..13).rev() {
        if st.rank_counts[pair_rank] >= 2 {
            let mut kickers_desc = [0usize; 3];
            let mut n = 0usize;

            for rank in (0..13).rev() {
                if rank != pair_rank && st.rank_counts[rank] != 0 {
                    kickers_desc[n] = rank;
                    n += 1;
                    if n == 3 {
                        break;
                    }
                }
            }

            debug_assert!(n == 3);

            let payload =
                pair_payload(pair_rank, kickers_desc[2], kickers_desc[1], kickers_desc[0]);
            return Some(pack(CAT_PAIR, payload));
        }
    }

    None
}

// Evaluates the top five distinct ranks as high card
#[inline]
fn eval_high_card(st: &EvalState) -> HandRank {
    let top5 = top_n_mask(st.rank_mask, 5);
    let payload = rank5_payload_from_mask(top5);
    pack(CAT_HIGH_CARD, payload)
}
