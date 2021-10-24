//! # What is Skull?
//!
//! Skull is the quintessence of bluffing, a game in which everything is played
//! in the players' heads.
//! Each player plays a face-down card, then each player in turn adds one more
//! card – until someone feels safe enough to state that he can turn a number
//! of cards face up and get only roses.
//! Other players can then overbid him, saying they can turn even more cards
//! face up.
//! The highest bidder must then turn that number of cards face up, starting
//! with his own.
//! If he shows only roses, he wins; if he reveals a skull, he loses, placing
//! one of his cards out of play.
//! Two successful challenges wins the game.
//! Skull is not a game of luck; it's a game of poker face and meeting eyes.
//!
//! (Edited description from Bruno Faidutti's write-up of the game in his
//! [Ideal Game Library](http://www.faidutti.com/index.php?Module=ludotheque&id=728))
//!
//! ## How do I play Skull?
//!
//! Here's the [game's manual](http://www.skull-and-roses.com/pdf/Skull_EnP.pdf)
//! (in English)
//!
//! # What does this crate provide?
//!
//! This crate provides a **simulation** of the game Skull.
//! It allows for the creation of a [`Game`] for 3 to 6 players (as recommended
//! by the original) and provides all necessary means to interact with the
//! game and understand the current state of the game.
//! It enforces all of the games rules and scoring for you, so you only need
//! to focus on how you wish to present the game.
//!
//! Please note the documentation has been written on the assumption of an
//! understanding of the way Skull works.
//! If you don't know, it is highly recommended to read the manual and play the
//! game at least once to grasp it.
//!

#![warn(missing_docs)]

mod game;
mod hand;

pub use heapless::Vec as FVec; // Fixed Vec

use std::convert::TryFrom;
use std::fmt;

pub use rand::rngs::ThreadRng;
use rand::Rng;

use Card::*;
use Event::*;
use State::*;

#[doc(inline)]
pub use game::Game;
#[doc(inline)]
pub use hand::Hand;

type OrderedHand = FVec<Card, 4>;

#[doc(hidden)]
#[macro_export]
macro_rules! fvec {
    () => {
        heapless::Vec::new()
    };
    ($elem:expr; $n:expr) => {
        heapless::Vec::from_slice(&[$elem; $n]).unwrap()
    };
    ( $( $x:expr ),* ) => {
        heapless::Vec::from_slice(&[$($x),*]).unwrap()
    };
}

/// A playing card
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Card {
    #[allow(missing_docs)]
    Flower,
    #[allow(missing_docs)]
    Skull,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Reuse Debug impl
        write!(f, "{:?}", self)
    }
}

/// Describes the current state of play of the game
///
/// Skulls cycles through three phases of play:
/// 1. Playing (putting down cards)
/// 2. Bidding (determining a number of cards to challenge for)
/// 3. Challenging (trying to turn over the chosen number of flowers)
///
/// This enum has a variant for each state, each of which holds any additional
/// information relevant only to that state
///
/// State is generic over the number of players
///
/// It is expected that you would only ever get a State by calling
/// [`Game::state()`], instead of creating one
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum State<const N: usize> {
    /// When players are putting down cards
    Playing {
        /// The index of the current player
        current_player: usize,
    },
    /// When players are determining a number of cards to challenge for
    Bidding {
        /// The index of the current bidder
        current_bidder: usize,
        /// The current highest bid (a number of cards)
        highest_bid: usize,
        /// The index of the highest bidder
        highest_bidder: usize,
        /// The highest bid possible (total number of cards played)
        max_bid: usize,
        /// Keeps track of the players who have passed
        passed: [bool; N],
    },
    /// When a player is trying to turn over the chosen number of flowers
    Challenging {
        /// The index of the challenger
        challenger: usize,
        /// The number of flowers the challenger is trying to flip
        target: usize,
        /// The per-player indexes of flipped cards
        ///
        /// For the challenger, the indexes will always be ordered from low to
        /// high as the cards are automatically flipped for them
        flipped: [FVec<usize, 4>; N],
    },
}

/// An event is either a notification of something important, or a prompt to
/// tell the user/programmer what input is expected
///
/// You can think of them a bit like a notification (except `Input`)
///
/// You are **required** to process all pending events before giving the game
/// another input.
/// See [`Game::what_next()`] for more information
///
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Event {
    /// Indicates that an input is required from one of the game's players
    Input {
        /// Which player (by index) should be making the input
        player: usize,
        /// What sort of input the game is expecting
        input: InputType,
    },
    /// Notifies that a bid has started and the game [`State`] has changed
    BidStarted,
    /// Notifies that a challenge has started and the game [`State`] has
    /// changed
    ChallengeStarted,
    /// Notifies that a challenge has ended because the challenger flipped a
    /// skull.
    /// The game [`State`] has changed
    ChallengerChoseSkull {
        /// The index of the challenger
        challenger: usize,
        /// The index of the skull player
        /// (can potentially be the same as `challenger`)
        skull_player: usize,
    },
    /// Notifies that a player has lost all their cards
    /// (index of player provided)
    PlayerOut(usize),
    /// Notifies that the challenger won their challenge
    /// (index of challenger provided)
    ChallengeWon(usize),
    /// Notifies that the challenger won their challenge and has now won the
    /// game (index of winner provided)
    ChallengeWonGameWon(usize),
}

/// The type of input required from the player
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputType {
    /// The player must play a card
    PlayCard,           // When not everyone has played a card
    /// The player can either play a card or start a bid
    PlayCardOrStartBid, // When everyone has played a card
    /// The player must start a bid
    StartBid,           // When player has no cards remaining
    /// The player must bid or pass
    BidOrPass,
    /// The challenger must flip a card
    FlipCard,
}

/// The type of input given to the game using [`Game::respond()`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Response {
    /// The current player plays the specified card
    PlayCard(Card),
    /// The current bidder raises the bid to the specified number
    Bid(usize),
    /// The current bidder opts to pass
    Pass,
    /// The challenger flips over a card `(player_index, card_index)`
    Flip(usize, usize),
}

/// The type of error produced by [`Game::respond()`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ResponseError {
    /// Can't take an input now because there is another [`Event`] that needs
    /// processing.
    /// Call [`Game::what_next()`]
    PendingEvent,
    /// Input type didn't match what was expected.
    /// Correct [`InputType`] provided
    IncorrectInputType(InputType),
    /// Player tried to put down a card they don't have
    /// (either already played, or discarded)
    CardNotInHand,
    /// Bid submitted was lower than current bid.
    /// Minimum acceptable bid provided
    BidTooLow(usize),
    /// Bid submitted was too high (in excess of the number of cards played).
    /// Maximum acceptable bid provided
    BidTooHigh(usize),
    /// Out of range index given when challenger tried to flip a card
    InvalidIndex,
    /// Challenger is trying to flip a card they've already flipped
    CardAlreadyFlipped,
    /// Challenger is trying to flip their own cards
    /// (they're flipped automatically)
    ManuallyFlippingOwnCards,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InputType::*;
        use ResponseError::*;
        match self {
            PendingEvent => write!(f, "There's a pending event that needs to be processed using Game.what_next()"),
            IncorrectInputType(it) => {
                write!(f, "Incorrect input type, expected {}", match it {
                    PlayCard => "PlayCard",
                    PlayCardOrStartBid => "PlayCard or Bid",
                    StartBid => "Bid",
                    BidOrPass => "Bid or Pass",
                    FlipCard => "Flip",
                })
            }
            CardNotInHand => write!(f, "The player doesn't have that card"),
            BidTooLow(min) => write!(f, "Bid too low, needs to be at least {}", min),
            BidTooHigh(max) => write!(f, "Bid too high, needs to be at most {}", max),
            InvalidIndex => write!(f, "Invalid index, outside of allowed range"),
            CardAlreadyFlipped => write!(f, "The player has already flipped that card"),
            ManuallyFlippingOwnCards => write!(f, "Challenger is trying to flip their own cards, which are already flipped"),
        }
    }
}

impl std::error::Error for ResponseError {}
