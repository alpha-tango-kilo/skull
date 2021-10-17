use smallvec::{smallvec, SmallVec};

use State::*;

#[derive(Debug, Copy, Clone)]
pub enum Card {
    Flower,
    Skull,
}

#[derive(Debug, Default, Copy, Clone)]
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

    pub fn count(&self) -> u8 {
        self.flowers + self.skull as u8
    }

    pub fn as_vec(&self) -> Vec<Card> {
        let mut v = vec![Card::Flower; self.flowers as usize];
        if self.skull { v.insert(0, Card::Skull) }
        v
    }

    fn is_superset_of(&self, other: Hand) -> bool {
        let skull_ok = self.skull == other.skull || (self.skull && !other.skull);
        let flowers_ok = self.flowers >= other.flowers;
        skull_ok && flowers_ok
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
             F       T        Err (covered above)
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

#[allow(clippy::large_enum_variant)] // Might fix this later
#[derive(Debug)]
pub enum State {
    Playing {
        current_player: usize,
    },
    Bidding {
        current_bidder: usize,
        current_bid: usize,
        max_bid: usize,
        passed: SmallVec<[bool; 6]>,
    },
    Challenging {
        challenger: usize,
        target: usize,
        flipped: SmallVec<[SmallVec<[usize; 4]>; 6]>,
    },
}

impl Default for State {
    fn default() -> Self {
        Playing { current_player: 0 }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InputRequest {
    pub player: usize,
    pub input: InputType,
}

#[derive(Debug, Copy, Clone)]
pub enum InputType {
    PlayCard,           // When not everyone has played a card
    PlayCardOrStartBid, // When everyone has played a card
    BidOrPass,
    FlipCard,
}

#[derive(Debug)]
pub struct Game {
    scores: SmallVec<[u8; 6]>,
    player_hands: SmallVec<[Hand; 6]>,
    cards_played: SmallVec<[Hand; 6]>,
    state: State,
}

impl Game {
    pub fn new(players: usize) -> Self {
        // Range should preferably be checked by wrapper (CLI/GUI)
        assert!((3..=6).contains(&players), "Invalid number of players");

        Game {
            scores: smallvec![0; players],
            player_hands: smallvec![Hand::new(); players],
            cards_played: smallvec![Default::default(); players],
            state: Default::default(),
        }
    }

    pub fn scores(&self) -> &[u8] {
        self.scores.as_slice()
    }

    pub fn hands(&self) -> &[Hand] {
        self.player_hands.as_slice()
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn next_input(&self) -> InputRequest {
        use InputType::*;
        let player = self.player();

        match self.state {
            Playing { .. } => {
                if self.cards_played_count() as usize >= self.player_hands.len() {
                    InputRequest {
                        player,
                        input: PlayCardOrStartBid,
                    }
                } else {
                    InputRequest {
                        player,
                        input: PlayCard,
                    }
                }
            }
            Bidding { .. } => InputRequest {
                player,
                input: BidOrPass,
            },
            Challenging { .. } => InputRequest {
                player,
                input: FlipCard,
            },
        }
    }

    fn player(&self) -> usize {
        match self.state {
            Playing { current_player } => current_player,
            Bidding { current_bidder, .. } => current_bidder,
            Challenging { challenger, .. } => challenger,
        }
    }

    fn cards_played_count(&self) -> u8 {
        self.cards_played.iter().map(Hand::count).sum()
    }
}
