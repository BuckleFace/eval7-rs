/// Poker hand categories ordered from weakest to strongest.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HandCategory {
    /// Five unmatched cards that form no stronger category.
    HighCard = 0,
    /// Two cards of one rank.
    Pair = 1,
    /// Two cards of one rank and two cards of another rank.
    TwoPair = 2,
    /// Three cards of one rank.
    Trips = 3,
    /// Five consecutive ranks.
    Straight = 4,
    /// Five cards of one suit.
    Flush = 5,
    /// Three cards of one rank and two cards of another rank.
    FullHouse = 6,
    /// Four cards of one rank.
    Quads = 7,
    /// Five consecutive ranks of one suit.
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
    ///
    /// The high four bits encode the [`HandCategory`] discriminant; the low
    /// twelve bits contain the category-local ordering payload. This encoding
    /// is unchanged in crate version 0.2.0.
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Returns the category-local ordering payload.
    ///
    /// Larger payloads are stronger within a category. Payloads from different
    /// categories are not directly comparable; compare the full rank instead.
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

#[cfg(test)]
pub(crate) const fn pack(category: u16, payload: u16) -> HandRank {
    HandRank((category << CATEGORY_SHIFT) | payload)
}
