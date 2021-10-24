//! # What is Skull?
//!
//! Skull is the quintessence of bluffing, a game in which everything is played in the players' heads.
//! Each player plays a face-down card, then each player in turn adds one more card – until someone feels safe enough to state that he can turn a number of cards face up and get only roses.
//! Other players can then overbid him, saying they can turn even more cards face up.
//! The highest bidder must then turn that number of cards face up, starting with his own.
//! If he shows only roses, he wins; if he reveals a skull, he loses, placing one of his cards out of play.
//! Two successful challenges wins the game.
//! Skull is not a game of luck; it's a game of poker face and meeting eyes.
//!
//! (Edited description from Bruno Faidutti's write-up of the game in his [Ideal Game Library](http://www.faidutti.com/index.php?Module=ludotheque&id=728))
//!
//! ## How do I play Skull?
//!
//! Here's the [game's manual](http://www.skull-and-roses.com/pdf/Skull_EnP.pdf) (in English)
//!
//! # What does this crate provide?
//!
//! This crate provides a **simulation** of the game Skull.
//! It allows for the creation of a [Game] for 3 to 6 players (as recommended by the original) and provides all necessary means to interact with the game and understand the current state of the game.
//! It enforces all of the games rules and scoring for you, so you only need to focus on how you wish to present the game.
//!
//! Please note the documentation has been written on the assumption of an understand of the way Skull works.
//! If you don't know, it is highly recommended to read the manual and play the game at least once to grasp it.
//!

#![warn(missing_docs)]

mod game;
mod hand;

use heapless::Vec as FVec; // Fixed Vec

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum State<const N: usize> {
    Playing {
        current_player: usize,
    },
    Bidding {
        current_bidder: usize,
        highest_bid: usize,
        highest_bidder: usize,
        max_bid: usize,
        passed: [bool; N],
    },
    Challenging {
        challenger: usize,
        target: usize,
        flipped: [FVec<usize, 4>; N], // This is sorted for own cards (low - high)
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Event {
    Input {
        player: usize,
        input: InputType,
    },
    BidStarted,
    ChallengeStarted,
    ChallengerChoseSkull {
        challenger: usize,
        skull_player: usize,
    },
    PlayerOut(usize),
    ChallengeWon(usize),
    ChallengeWonGameWon(usize),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputType {
    PlayCard,           // When not everyone has played a card
    PlayCardOrStartBid, // When everyone has played a card
    StartBid,           // When player has no cards remaining
    BidOrPass,
    FlipCard,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Response {
    PlayCard(Card),
    Bid(usize),
    Pass,
    Flip(usize, usize),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ResponseError {
    PendingEvent,
    IncorrectInputType(InputType),
    CardNotInHand,
    BidTooLow(usize),
    BidTooHigh(usize),
    InvalidIndex,
    CardAlreadyFlipped,
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
