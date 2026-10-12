#![allow(clippy::collapsible_if)]

use crate::Card;

use crate::rank::{
    CAT_FLUSH, CAT_FULL_HOUSE, CAT_HIGH_CARD, CAT_PAIR, CAT_QUADS, CAT_STRAIGHT,
    CAT_STRAIGHT_FLUSH, CAT_TRIPS, CAT_TWO_PAIR, HandRank, pack,
};
use crate::tables::CHOOSE;

// Reference 5-card eval result used by tests
// Carries both the comparable rank and the chosen 5-card set
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RefEval5 {
    pub(super) rank: HandRank,
    pub(super) best_five: [Card; 5],
}

// Reference evaluator for exactly 5 cards
pub(super) fn reference_eval_5(cards: [Card; 5]) -> RefEval5 {
    let rank = eval_5_rank(cards);
    RefEval5 {
        rank,
        best_five: cards,
    }
}

// Reference evaluator for 7 cards
// Enumerates all 21 five-card subsets and keeps the best one
pub(super) fn reference_eval_7(cards: [Card; 7]) -> RefEval5 {
    let mut best = reference_eval_5(pick5(cards, FIVE_CARD_SUBSETS[0]));

    for subset in FIVE_CARD_SUBSETS.iter().copied().skip(1) {
        let candidate = reference_eval_5(pick5(cards, subset));
        if candidate.rank > best.rank {
            best = candidate;
        }
    }

    best
}

// All 21 ways to choose 5 cards from 7
const FIVE_CARD_SUBSETS: [[usize; 5]; 21] = [
    [0, 1, 2, 3, 4],
    [0, 1, 2, 3, 5],
    [0, 1, 2, 3, 6],
    [0, 1, 2, 4, 5],
    [0, 1, 2, 4, 6],
    [0, 1, 2, 5, 6],
    [0, 1, 3, 4, 5],
    [0, 1, 3, 4, 6],
    [0, 1, 3, 5, 6],
    [0, 1, 4, 5, 6],
    [0, 2, 3, 4, 5],
    [0, 2, 3, 4, 6],
    [0, 2, 3, 5, 6],
    [0, 2, 4, 5, 6],
    [0, 3, 4, 5, 6],
    [1, 2, 3, 4, 5],
    [1, 2, 3, 4, 6],
    [1, 2, 3, 5, 6],
    [1, 2, 4, 5, 6],
    [1, 3, 4, 5, 6],
    [2, 3, 4, 5, 6],
];

// Picks one 5-card subset from a 7-card array
fn pick5(cards: [Card; 7], idx: [usize; 5]) -> [Card; 5] {
    [
        cards[idx[0]],
        cards[idx[1]],
        cards[idx[2]],
        cards[idx[3]],
        cards[idx[4]],
    ]
}

// Evaluates exactly 5 cards into the canonical packed rank format
fn eval_5_rank(cards: [Card; 5]) -> HandRank {
    let analysis = analyze_5(cards);

    if analysis.is_flush {
        if let Some(high) = analysis.straight_high {
            return pack(CAT_STRAIGHT_FLUSH, straight_payload(high));
        }
    }

    if let Some((quad_rank, kicker)) = analysis.quads() {
        return pack(CAT_QUADS, quads_payload(quad_rank, kicker));
    }

    if let Some((trips_rank, pair_rank)) = analysis.full_house() {
        return pack(CAT_FULL_HOUSE, full_house_payload(trips_rank, pair_rank));
    }

    if analysis.is_flush {
        return pack(
            CAT_FLUSH,
            rank5_payload_from_desc_ranks(analysis.desc_ranks),
        );
    }

    if let Some(high) = analysis.straight_high {
        return pack(CAT_STRAIGHT, straight_payload(high));
    }

    if let Some((trips_rank, kickers)) = analysis.trips() {
        return pack(CAT_TRIPS, trips_payload(trips_rank, kickers[1], kickers[0]));
    }

    if let Some((high_pair, low_pair, kicker)) = analysis.two_pair() {
        return pack(CAT_TWO_PAIR, two_pair_payload(high_pair, low_pair, kicker));
    }

    if let Some((pair_rank, kickers)) = analysis.pair() {
        return pack(
            CAT_PAIR,
            pair_payload(pair_rank, kickers[2], kickers[1], kickers[0]),
        );
    }

    pack(
        CAT_HIGH_CARD,
        rank5_payload_from_desc_ranks(analysis.desc_ranks),
    )
}

// Compact derived state for reference 5-card evaluation
#[derive(Debug, Clone, Copy)]
struct Analysis5 {
    rank_counts: [u8; 13],
    desc_ranks: [usize; 5],
    is_flush: bool,
    straight_high: Option<usize>,
}

impl Analysis5 {
    // Returns quads rank plus kicker if present
    fn quads(&self) -> Option<(usize, usize)> {
        let quad_rank = (0..13).rev().find(|&rank| self.rank_counts[rank] == 4)?;
        let kicker = self
            .desc_ranks
            .into_iter()
            .find(|&rank| rank != quad_rank)?;
        Some((quad_rank, kicker))
    }

    // Returns trips rank plus pair rank if present
    fn full_house(&self) -> Option<(usize, usize)> {
        let trips_rank = (0..13).rev().find(|&rank| self.rank_counts[rank] == 3)?;
        let pair_rank = (0..13).rev().find(|&rank| self.rank_counts[rank] == 2)?;
        Some((trips_rank, pair_rank))
    }

    // Returns trips rank plus the two kickers if present
    fn trips(&self) -> Option<(usize, [usize; 2])> {
        let trips_rank = (0..13).rev().find(|&rank| self.rank_counts[rank] == 3)?;
        let kickers: Vec<usize> = self
            .desc_ranks
            .into_iter()
            .filter(|&rank| rank != trips_rank)
            .collect();

        debug_assert!(kickers.len() == 2);
        Some((trips_rank, [kickers[0], kickers[1]]))
    }

    // Returns high pair, low pair, and kicker if present
    fn two_pair(&self) -> Option<(usize, usize, usize)> {
        let pairs: Vec<usize> = (0..13)
            .rev()
            .filter(|&rank| self.rank_counts[rank] == 2)
            .collect();

        if pairs.len() != 2 {
            return None;
        }

        let kicker = self
            .desc_ranks
            .into_iter()
            .find(|&rank| rank != pairs[0] && rank != pairs[1])?;

        Some((pairs[0], pairs[1], kicker))
    }

    // Returns pair rank plus the three kickers if present
    fn pair(&self) -> Option<(usize, [usize; 3])> {
        let pair_rank = (0..13).rev().find(|&rank| self.rank_counts[rank] == 2)?;
        let kickers: Vec<usize> = self
            .desc_ranks
            .into_iter()
            .filter(|&rank| rank != pair_rank)
            .collect();

        debug_assert!(kickers.len() == 3);
        Some((pair_rank, [kickers[0], kickers[1], kickers[2]]))
    }
}

// Analyzes 5 cards into rank counts, rank ordering, flush state, and straight state
fn analyze_5(cards: [Card; 5]) -> Analysis5 {
    let mut rank_counts = [0u8; 13];
    let mut suit_counts = [0u8; 4];
    let mut distinct_mask = 0u16;

    for card in cards {
        let id = card.id() as usize;
        debug_assert!(id < 52);

        let rank = id % 13;
        let suit = id / 13;

        rank_counts[rank] += 1;
        suit_counts[suit] += 1;
        distinct_mask |= 1u16 << rank;
    }

    let mut desc_ranks = [0usize; 5];
    let mut out = 0usize;
    for rank in (0..13).rev() {
        for _ in 0..rank_counts[rank] {
            desc_ranks[out] = rank;
            out += 1;
        }
    }
    debug_assert!(out == 5);

    Analysis5 {
        rank_counts,
        desc_ranks,
        is_flush: suit_counts.into_iter().any(|count| count == 5),
        straight_high: straight_high_from_mask(distinct_mask),
    }
}

// Returns the straight high rank index from a 13-bit rank mask
// Returns 3 for the wheel and None when no straight exists
fn straight_high_from_mask(mask: u16) -> Option<usize> {
    for high in (4..13).rev() {
        let need = (1u16 << high)
            | (1u16 << (high - 1))
            | (1u16 << (high - 2))
            | (1u16 << (high - 3))
            | (1u16 << (high - 4));

        if (mask & need) == need {
            return Some(high);
        }
    }

    let wheel = (1u16 << 12) | (1u16 << 0) | (1u16 << 1) | (1u16 << 2) | (1u16 << 3);

    if (mask & wheel) == wheel {
        Some(3)
    } else {
        None
    }
}

// Converts straight high rank index into the packed straight payload ordinal
fn straight_payload(high_rank_idx: usize) -> u16 {
    match high_rank_idx {
        3 => 0,
        4..=12 => (high_rank_idx - 3) as u16,
        _ => panic!("invalid straight high rank index: {high_rank_idx}"),
    }
}

// Encodes five descending ranks into the canonical 5-rank payload
fn rank5_payload_from_desc_ranks(desc_ranks: [usize; 5]) -> u16 {
    let mut asc = desc_ranks;
    asc.reverse();

    CHOOSE[asc[0]][1]
        + CHOOSE[asc[1]][2]
        + CHOOSE[asc[2]][3]
        + CHOOSE[asc[3]][4]
        + CHOOSE[asc[4]][5]
}

// Removes one excluded rank from ordinal space
fn compress1(rank: usize, dead: usize) -> usize {
    if rank < dead { rank } else { rank - 1 }
}

// Removes two excluded ranks from ordinal space
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

// Encodes pair rank plus kickers into a pair payload
fn pair_payload(pair_rank: usize, k0: usize, k1: usize, k2: usize) -> u16 {
    let a = compress1(k0, pair_rank);
    let b = compress1(k1, pair_rank);
    let c = compress1(k2, pair_rank);

    let kicker_ordinal = CHOOSE[a][1] + CHOOSE[b][2] + CHOOSE[c][3];

    (pair_rank as u16) * 220 + kicker_ordinal
}

// Encodes two pair plus kicker into a two-pair payload
fn two_pair_payload(high_pair: usize, low_pair: usize, kicker: usize) -> u16 {
    debug_assert!(low_pair < high_pair);

    let pair_block = CHOOSE[low_pair][1] + CHOOSE[high_pair][2];

    let kicker_ordinal = compress2(kicker, low_pair, high_pair) as u16;

    pair_block * 11 + kicker_ordinal
}

// Encodes trips plus kickers into a trips payload
fn trips_payload(trips_rank: usize, k0: usize, k1: usize) -> u16 {
    let a = compress1(k0, trips_rank);
    let b = compress1(k1, trips_rank);

    let kicker_ordinal = CHOOSE[a][1] + CHOOSE[b][2];

    (trips_rank as u16) * 66 + kicker_ordinal
}

// Encodes quads plus kicker into a quads payload
fn quads_payload(quad_rank: usize, kicker: usize) -> u16 {
    let kicker_ordinal = compress1(kicker, quad_rank) as u16;
    (quad_rank as u16) * 12 + kicker_ordinal
}

// Encodes trips rank plus pair rank into a full-house payload
fn full_house_payload(trips_rank: usize, pair_rank: usize) -> u16 {
    let pair_ordinal = compress1(pair_rank, trips_rank) as u16;
    (trips_rank as u16) * 12 + pair_ordinal
}
