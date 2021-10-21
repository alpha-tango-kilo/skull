mod game;

use heapless::Vec as FVec; // Fixed Vec

use std::convert::TryFrom;
use std::fmt;

use rand::rngs::ThreadRng;
use rand::Rng;

use Card::*;
use Event::*;
use State::*;

pub use game::Game;

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

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Hand {
    skull: bool,
    flowers: u8,
}

impl Hand {
    // WARNING: new() and default() differ
    pub fn new() -> Self {
        Hand {
            skull: true,
            flowers: 3,
        }
    }

    pub fn has_skull(&self) -> bool {
        self.skull
    }

    pub fn has(&self, other: Card) -> bool {
        use Card::*;
        match other {
            Skull => self.has_skull(),
            Flower => self.flowers > 0,
        }
    }

    pub fn count(&self) -> u8 {
        self.flowers + self.skull as u8
    }

    pub fn empty(&self) -> bool {
        self.count() == 0
    }

    pub fn as_vec(&self) -> Vec<Card> {
        let mut v = vec![Card::Flower; self.flowers as usize];
        if self.skull {
            v.insert(0, Card::Skull)
        }
        v
    }

    pub fn can_play(&self) -> Hand {
        // Assumes self is a valid hand
        (Hand::new() - *self).unwrap()
    }

    fn is_superset_of(&self, other: Hand) -> bool {
        let skull_ok =
            self.skull == other.skull || (self.skull && !other.skull);
        let flowers_ok = self.flowers >= other.flowers;
        skull_ok && flowers_ok
    }

    fn discard_one(&mut self, rng: &mut ThreadRng) {
        debug_assert!(
            self.count() > 0,
            "Tried to discard card with none in hand"
        );

        let choice = rng.gen_range(0..=self.count());
        if choice == 0 && self.skull {
            self.skull = false;
        } else {
            self.flowers -= 1;
        }
    }

    pub(crate) fn assert_valid(&self) {
        assert!(self.flowers < 4, "Too many flowers in hand");
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_vec())
    }
}

impl std::ops::Sub for Hand {
    type Output = Result<Hand, &'static str>;

    fn sub(self, rhs: Self) -> Self::Output {
        if !self.is_superset_of(rhs) {
            Err("RHS has cards LHS doesn't have")
        } else {
            /*
            Truth table for skull:
            LHS     RHS     Output
             F       F        F
             F       T        Err
             T       F        T
             T       T        F
            Because the Err condition has already been checked, we can just XOR (^) here
             */
            let skull = self.skull ^ rhs.skull;
            // Subtraction doesn't need to be checked because of check
            let flowers = self.flowers - rhs.flowers;
            Ok(Hand { skull, flowers })
        }
    }
}

impl TryFrom<&[Card]> for Hand {
    type Error = &'static str;

    fn try_from(value: &[Card]) -> Result<Self, Self::Error> {
        let mut skull = false;
        let mut flowers = 0;
        for n in value {
            use Card::*;
            match n {
                Skull => {
                    if !skull {
                        skull = true
                    } else {
                        return Err("Multiple skulls");
                    }
                }
                Flower => {
                    if flowers < 3 {
                        flowers += 1
                    } else {
                        return Err("Too many flowers");
                    }
                }
            }
        }
        Ok(Hand { skull, flowers })
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
    PlayerOut(usize), // TODO: implement this
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
        }
    }
}

impl std::error::Error for RespondError {}
