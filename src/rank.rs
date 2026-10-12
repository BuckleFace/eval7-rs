/// Poker hand categories ordered from weakest to strongest.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HandCategory {
    HighCard = 0,
    Pair = 1,
    TwoPair = 2,
    Trips = 3,
    Straight = 4,
    Flush = 5,
    FullHouse = 6,
    Quads = 7,
    StraightFlush = 8,
}

/// A compact, directly comparable poker hand rank.
///
/// Larger values represent stronger hands. Equality means the best five-card
/// poker hands tie exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HandRank(u16);

pub(crate) const CATEGORY_SHIFT: u16 = 12;
pub(crate) const PAYLOAD_MASK: u16 = 0x0fff;

pub(crate) const CAT_HIGH_CARD: u16 = HandCategory::HighCard as u16;
pub(crate) const CAT_PAIR: u16 = HandCategory::Pair as u16;
pub(crate) const CAT_TWO_PAIR: u16 = HandCategory::TwoPair as u16;
pub(crate) const CAT_TRIPS: u16 = HandCategory::Trips as u16;
pub(crate) const CAT_STRAIGHT: u16 = HandCategory::Straight as u16;
pub(crate) const CAT_FLUSH: u16 = HandCategory::Flush as u16;
pub(crate) const CAT_FULL_HOUSE: u16 = HandCategory::FullHouse as u16;
pub(crate) const CAT_QUADS: u16 = HandCategory::Quads as u16;
pub(crate) const CAT_STRAIGHT_FLUSH: u16 = HandCategory::StraightFlush as u16;

impl HandRank {
    /// Returns the stable v1 packed rank value.
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Returns the category-local ordering payload.
    pub const fn payload(self) -> u16 {
        self.0 & PAYLOAD_MASK
    }

    /// Returns the semantic hand category.
    pub const fn category(self) -> HandCategory {
        match self.0 >> CATEGORY_SHIFT {
            CAT_HIGH_CARD => HandCategory::HighCard,
            CAT_PAIR => HandCategory::Pair,
            CAT_TWO_PAIR => HandCategory::TwoPair,
            CAT_TRIPS => HandCategory::Trips,
            CAT_STRAIGHT => HandCategory::Straight,
            CAT_FLUSH => HandCategory::Flush,
            CAT_FULL_HOUSE => HandCategory::FullHouse,
            CAT_QUADS => HandCategory::Quads,
            CAT_STRAIGHT_FLUSH => HandCategory::StraightFlush,
            _ => unreachable!(),
        }
    }

    pub(crate) const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }
}

#[allow(dead_code)]
pub(crate) const fn pack(category: u16, payload: u16) -> HandRank {
    HandRank((category << CATEGORY_SHIFT) | payload)
}
