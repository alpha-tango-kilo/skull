mod game;
mod hand;

use heapless::Vec as FVec; // Fixed Vec

use std::convert::TryFrom;
use std::fmt;

use rand::rngs::ThreadRng;
use rand::Rng;

use Card::*;
use Event::*;
use State::*;

pub use game::Game;
pub use hand::Hand;

type OrderedHand = FVec<Card, 4>;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Card {
    Flower,
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
pub enum RespondError {
    PendingEvent,
    IncorrectInputType(InputType),
    CardNotInHand,
    CardAlreadyFlipped,
}

impl fmt::Display for RespondError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InputType::*;
        use RespondError::*;
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
            CardAlreadyFlipped => write!(f, "The player has already flipped that card"),
        }
    }
}

impl std::error::Error for RespondError {}
