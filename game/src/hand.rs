use std::ops::Sub;

use crate::*;

use std::error::Error;
use HandError::*;

/// A player's hand when playing Skull
///
/// Can contain at most one [skull](Card::Skull), and at most 3
/// [flowers](Card::Flower)
///

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Hand {
    skull: bool,
    flowers: u8,
}

impl Hand {
    /// Creates a new full hand (1 skull, 3 flowers)
    // WARNING: new() and default() differ
    pub const fn new() -> Self {
        Hand {
            skull: true,
            flowers: 3,
        }
    }

    /// Returns `true` if there is a skull in the hand
    pub const fn has_skull(&self) -> bool {
        self.skull
    }

    /// Returns `true` if the given card is present in the hand
    pub const fn has(&self, other: Card) -> bool {
        use Card::*;
        match other {
            Skull => self.has_skull(),
            Flower => self.flowers > 0,
        }
    }

    /// Returns the number of cards in the hand, as a [u8]
    pub const fn count(&self) -> u8 {
        self.flowers + self.skull as u8
    }

    /// Returns true if there are no cards in the hand
    pub const fn empty(&self) -> bool {
        self.count() == 0
    }

    /// Represents the hand as a `Vec<Card>`
    ///
    /// Useful for when you want to display the cards in a hand
    ///
    /// This method allocates a new `Vec`
    pub fn as_vec(&self) -> Vec<Card> {
        let mut v = vec![Card::Flower; self.flowers as usize];
        if self.skull {
            v.insert(0, Card::Skull)
        }
        v
    }

    /// Returns `true` if `other` is a subset of `self`
    pub(crate) fn is_superset_of(&self, other: Hand) -> bool {
        let skull_ok =
            self.skull == other.skull || (self.skull && !other.skull);
        let flowers_ok = self.flowers >= other.flowers;
        skull_ok && flowers_ok
    }

    /// Discards a single random card from the hand
    pub(crate) fn discard_one(&mut self, rng: &mut ThreadRng) {
        debug_assert!(
            self.count() > 0,
            "Tried to discard card with none in hand"
        );

        if self.skull && self.flowers > 0 {
            let choice = rng.gen_range(0..=self.count());
            if choice == 0 {
                self.skull = false;
            } else {
                self.flowers -= 1;
            }
        } else if self.skull {
            self.skull = false
        } else {
            self.flowers -= 1;
        }
    }

    /// Checks the hand doesn't have an invalid number of flowers
    /// (i.e. more than 4)
    pub(crate) fn assert_valid(&self) {
        assert!(self.flowers < 4, "Too many flowers in hand");
    }
}

// Manually derived so the documentation is editable
impl Default for Hand {
    /// Creates a new empty hand
    fn default() -> Self {
        Hand {
            skull: false,
            flowers: 0,
        }
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_vec())
    }
}

impl TryFrom<&[Card]> for Hand {
    type Error = HandError;

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
                        return Err(MultipleSkulls);
                    }
                }
                Flower => {
                    if flowers < 3 {
                        flowers += 1
                    } else {
                        return Err(TooManyFlowers);
                    }
                }
            }
        }
        Ok(Hand { skull, flowers })
    }
}

impl<const N: usize> TryFrom<[Card; N]> for Hand {
    type Error = HandError;

    fn try_from(value: [Card; N]) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl Sub<Self> for Hand {
    type Output = Result<Hand, HandError>;

    fn sub(self, rhs: Self) -> Self::Output {
        if !self.is_superset_of(rhs) {
            Err(RhsNotSubset(self, rhs))
        } else {
            /*
            Truth table for skull:
            LHS     RHS     Output
             F       F        F
             F       T        Err
             T       F        T
             T       T        F
            Because the Err condition has already been checked, we can just XOR
            (^) here
             */
            let skull = self.skull ^ rhs.skull;
            // Subtraction doesn't need to be checked because of check
            let flowers = self.flowers - rhs.flowers;
            Ok(Hand { skull, flowers })
        }
    }
}

impl Sub<&[Card]> for Hand {
    type Output = Result<Hand, HandError>;

    fn sub(self, rhs: &[Card]) -> Self::Output {
        let rhs = Hand::try_from(rhs)?;
        self - rhs
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HandError {
    MultipleSkulls,
    TooManyFlowers,
    RhsNotSubset(Hand, Hand),
}

impl fmt::Display for HandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use HandError::*;
        match self {
            MultipleSkulls => write!(f, "Invalid hand, multiple skulls"),
            TooManyFlowers => write!(f, "Invalid hand, too many flowers"),
            RhsNotSubset(a, b) => write!(f, "RHS of subtraction had cards the left side didn't. Left: {}. Right: {}", a, b),
        }
    }
}

impl Error for HandError {}
