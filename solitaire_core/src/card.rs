use serde::{Deserialize, Serialize};

/// Card suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    /// Returns `true` for red suits (Diamonds, Hearts).
    pub fn is_red(self) -> bool {
        matches!(self, Suit::Diamonds | Suit::Hearts)
    }

    /// Returns `true` for black suits (Clubs, Spades).
    pub fn is_black(self) -> bool {
        !self.is_red()
    }
}

/// Card rank, Ace through King.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    /// Numeric value: Ace = 1, King = 13.
    pub fn value(self) -> u8 {
        match self {
            Rank::Ace   => 1,
            Rank::Two   => 2,
            Rank::Three => 3,
            Rank::Four  => 4,
            Rank::Five  => 5,
            Rank::Six   => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine  => 9,
            Rank::Ten   => 10,
            Rank::Jack  => 11,
            Rank::Queen => 12,
            Rank::King  => 13,
        }
    }
}

/// A single playing card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    /// Unique identifier for this card within the deal. Stable across moves and undo.
    pub id: u32,
    /// The card's suit (Clubs, Diamonds, Hearts, Spades).
    pub suit: Suit,
    /// The card's rank (Ace through King).
    pub rank: Rank,
    /// Whether the card is visible to the player. Face-down cards may not be moved.
    pub face_up: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_values_are_sequential() {
        let ranks = [
            Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five,
            Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen, Rank::King,
        ];
        for (i, r) in ranks.iter().enumerate() {
            assert_eq!(r.value(), (i + 1) as u8);
        }
    }

    #[test]
    fn suit_red_and_black_are_complementary() {
        for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            assert_ne!(suit.is_red(), suit.is_black(), "{suit:?} must be exactly one of red/black");
        }
        assert!(Suit::Diamonds.is_red() && Suit::Hearts.is_red());
        assert!(Suit::Clubs.is_black() && Suit::Spades.is_black());
    }
}
