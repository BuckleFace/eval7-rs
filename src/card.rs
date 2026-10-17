use std::fmt;
use std::str::FromStr;

/// An error returned when constructing or parsing a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardError {
    /// The numeric card ID is outside `0..=51`.
    InvalidId,
    /// The rank is outside `2..=14` or its text symbol is unrecognised.
    InvalidRank,
    /// The suit is outside `0..=3` or its text symbol is unrecognised.
    InvalidSuit,
    /// The textual representation does not contain exactly two characters.
    InvalidText,
}

impl fmt::Display for CardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidId => "card id must be in 0..=51",
            Self::InvalidRank => "card rank must be in 2..=14",
            Self::InvalidSuit => "card suit must be in 0..=3",
            Self::InvalidText => "card text must contain exactly one rank and one suit",
        };
        f.write_str(message)
    }
}

impl std::error::Error for CardError {}

/// A validated card from a standard 52-card deck.
///
/// IDs are suit-major: clubs, diamonds, hearts, spades, with ranks deuce
/// through ace within each suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card(u8);

impl Card {
    /// Creates a card from its canonical ID in `0..=51`.
    pub const fn new(id: u8) -> Result<Self, CardError> {
        if id < 52 {
            Ok(Self(id))
        } else {
            Err(CardError::InvalidId)
        }
    }

    /// Creates a card from a rank in `2..=14` and a suit in `0..=3`.
    ///
    /// Returns [`CardError::InvalidRank`] or [`CardError::InvalidSuit`] when
    /// the corresponding value is outside its accepted range.
    pub const fn from_rank_suit(rank: u8, suit: u8) -> Result<Self, CardError> {
        if rank < 2 || rank > 14 {
            return Err(CardError::InvalidRank);
        }
        if suit > 3 {
            return Err(CardError::InvalidSuit);
        }
        Ok(Self(suit * 13 + rank - 2))
    }

    /// Returns the canonical card ID.
    pub const fn id(self) -> u8 {
        self.0
    }

    /// Returns the rank in `2..=14`.
    pub const fn rank(self) -> u8 {
        self.0 % 13 + 2
    }

    /// Returns the suit in `0..=3`.
    pub const fn suit(self) -> u8 {
        self.0 / 13
    }

    /// Parses compact two-character card text such as `Ah` or `Td`.
    ///
    /// Rank letters and suit letters are ASCII case-insensitive. Invalid
    /// length, rank, and suit inputs return their corresponding [`CardError`].
    pub fn from_str2(text: &str) -> Result<Self, CardError> {
        let mut chars = text.chars();
        let rank = chars.next().ok_or(CardError::InvalidText)?;
        let suit = chars.next().ok_or(CardError::InvalidText)?;
        if chars.next().is_some() {
            return Err(CardError::InvalidText);
        }
        Self::from_rank_suit_chars(rank, suit)
    }

    const fn from_rank_suit_chars(rank: char, suit: char) -> Result<Self, CardError> {
        let rank = match rank {
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'T' | 't' => 10,
            'J' | 'j' => 11,
            'Q' | 'q' => 12,
            'K' | 'k' => 13,
            'A' | 'a' => 14,
            _ => return Err(CardError::InvalidRank),
        };
        let suit = match suit {
            'c' | 'C' => 0,
            'd' | 'D' => 1,
            'h' | 'H' => 2,
            's' | 'S' => 3,
            _ => return Err(CardError::InvalidSuit),
        };
        Self::from_rank_suit(rank, suit)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RANKS: &[u8; 13] = b"23456789TJQKA";
        const SUITS: &[u8; 4] = b"cdhs";
        let rank = RANKS[(self.0 % 13) as usize] as char;
        let suit = SUITS[(self.0 / 13) as usize] as char;
        write!(f, "{rank}{suit}")
    }
}

impl FromStr for Card {
    type Err = CardError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::from_str2(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_card_construction() {
        assert_eq!(Card::new(0).unwrap().to_string(), "2c");
        assert_eq!(Card::new(51).unwrap().to_string(), "As");
        assert_eq!(Card::new(52), Err(CardError::InvalidId));
        assert_eq!(Card::from_rank_suit(1, 0), Err(CardError::InvalidRank));
        assert_eq!(Card::from_rank_suit(14, 4), Err(CardError::InvalidSuit));
    }

    #[test]
    fn parses_compact_cards() {
        assert_eq!("Ah".parse::<Card>().unwrap().id(), 38);
        assert_eq!("td".parse::<Card>().unwrap().to_string(), "Td");
        assert_eq!("Ace".parse::<Card>(), Err(CardError::InvalidText));
    }
}
