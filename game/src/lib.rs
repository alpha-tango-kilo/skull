use smallvec::{smallvec, SmallVec};

use std::convert::TryFrom;
use std::fmt;

use rand::rngs::ThreadRng;
use rand::Rng;

use Event::*;
use State::*;

#[derive(Debug, Copy, Clone)]
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

    pub fn as_vec(&self) -> Vec<Card> {
        let mut v = vec![Card::Flower; self.flowers as usize];
        if self.skull {
            v.insert(0, Card::Skull)
        }
        v
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

#[allow(clippy::large_enum_variant)] // Might fix this later
#[derive(Debug)]
pub enum State {
    Playing {
        current_player: usize,
    },
    Bidding {
        current_bidder: usize,
        current_bid: usize,
        highest_bidder: usize,
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
pub enum Event {
    Input(InputRequest),
    BidStarted,
    ChallengeStarted,
    ChallengerChoseSkull {
        challenger: usize,
        skull_player: usize,
    },
    ChallengeWon(usize),
    ChallengeWonGameWon(usize),
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
    StartBid,           // When player has no cards remaining
    BidOrPass,
    FlipCard,
}

#[derive(Debug, Copy, Clone)]
pub enum Response {
    PlayCard(Card),
    Bid(usize),
    Pass,
    Flip(usize, usize),
}

#[derive(Debug)]
pub struct Game {
    scores: SmallVec<[u8; 6]>,          // public via getter
    player_hands: SmallVec<[Hand; 6]>,  // public via getter
    cards_played: SmallVec<[SmallVec<[Card; 4]>; 6]>,
    state: State,                       // public via getter
    pending_event: Option<Event>,
    rng: ThreadRng,
}

impl Game {
    pub fn new(players: usize) -> Self {
        assert!((3..=6).contains(&players), "Invalid number of players");

        Game {
            scores: smallvec![0; players],
            player_hands: smallvec![Hand::new(); players],
            cards_played: smallvec![Default::default(); players],
            state: Default::default(),
            pending_event: Default::default(),
            rng: Default::default(),
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

    pub fn what_next(&mut self) -> Event {
        use Event::*;
        use InputType::*;
        match self.pending_event {
            Some(event) => {
                match event {
                    ChallengeStarted => todo!("Check for own skulls"),
                    ChallengerChoseSkull { skull_player, .. } => {
                        // Transition back to playing
                        self.state = State::Playing { current_player: skull_player };
                    }
                    ChallengeWon(player) => {
                        // Transition back to playing
                        self.state = State::Playing { current_player: player };
                    }
                    Input(_) => unreachable!("Input events should never be stored as a pending event"),
                    _ => {} // No-ops: BidStarted, ChallengeWonGameWon
                }
                self.pending_event = None;
                event
            }
            None => Event::Input(InputRequest {
                player: self.player(),
                input: match self.state {
                    Playing { current_player } => {
                        // Check player has cards to play,
                        if self.cards_played[current_player].len()
                            < self.player_hands[current_player].count() as usize
                        {
                            // if they do, see if they're allowed to start bidding.
                            if self.cards_played_count() >= self.player_count()
                            {
                                PlayCardOrStartBid
                            } else {
                                PlayCard
                            }
                        } else {
                            // if they don't, they must start bidding.
                            StartBid
                        }
                    }
                    Bidding { .. } => BidOrPass,
                    Challenging { .. } => FlipCard,
                },
            }),
        }
    }

    pub fn respond(&mut self, response: &Response) {
        // These both have to be worked out before we start working mutably
        // with Game, even though they aren't always used
        let player_count = self.player_count();
        let played_count = self.cards_played_count();
        let flipped_count = self.cards_flipped_count();

        let Game { state, .. } = self;

        // TODO: starting a challenge (checking for own skulls)
        // TODO: account for players that are out
        // TODO: card discarding
        use Response::*;
        // Match against state & response instead of input?
        match (state, response) {
            // Playing card
            (Playing { current_player }, PlayCard(card)) => {
                assert!(
                    self.player_hands[*current_player].has(*card),
                    "Player is playing card they don't have\nHand: {}\nCard: {}",
                    self.player_hands[*current_player],
                    card
                );
                self.cards_played[*current_player].push(*card);
                self.increment_player();
            }
            // Starting bid
            (Playing { current_player }, Bid(n)) => {
                assert!(
                    *n <= played_count,
                    "Started bid for more cards than are in play"
                );
                if *n < played_count {
                    self.state = State::Bidding {
                        current_bidder: (*current_player + 1) % player_count,
                        current_bid: *n,
                        highest_bidder: *current_player,
                        max_bid: played_count,
                        passed: Default::default(),
                    };
                    self.pending_event = Some(BidStarted);
                } else {
                    // Start bid on max, instantly start challenge
                    self.state = State::Challenging {
                        challenger: *current_player,
                        target: played_count,
                        flipped: Default::default(),
                    };
                    self.pending_event = Some(ChallengeStarted);
                }
            }
            // Raising bid
            (
                Bidding {
                    current_bidder,
                    highest_bidder,
                    current_bid,
                    max_bid,
                    ..
                },
                Bid(n),
            ) => {
                assert!(
                    n <= max_bid,
                    "Bid greater than maximum ({} > {})",
                    n,
                    max_bid
                );
                assert!(
                    n > current_bid,
                    "Bid less than current ({} < {})",
                    n,
                    current_bid
                );
                *max_bid = *n;
                *highest_bidder = *current_bidder;
                self.increment_player();
                // TODO: check if bid is at max and start challenge if so
            }
            // Player passes on bid
            (Bidding { .. }, Pass) => {
                todo!("Passing on a bid (check if progressing to challenge)")
            }
            // Challenger flips a card
            (
                Challenging {
                    challenger,
                    target,
                    flipped,
                },
                Flip(player_index, card_index),
            ) => {
                assert!(
                    *player_index < player_count,
                    "Invalid player specified"
                );
                assert!(
                    *card_index
                        < self.player_hands[*player_index].count() as usize,
                    "Invalid card specified"
                );
                assert!(
                    !flipped[*player_index].contains(card_index),
                    "Tried to flip already-flipped card"
                );

                let card_flipped =
                    self.cards_played[*player_index][*card_index];
                use Card::*;
                match card_flipped {
                    Skull => {
                        self.player_hands[*challenger]
                            .discard_one(&mut self.rng);
                        self.pending_event = Some(ChallengerChoseSkull {
                            challenger: *challenger,
                            skull_player: *player_index,
                        });
                    }
                    Flower => {
                        flipped[*player_index].push(*card_index);
                        if flipped_count.unwrap() == *target {
                            self.scores[*challenger] += 1;
                            self.pending_event =
                                Some(if self.scores[*challenger] == 2 {
                                    ChallengeWonGameWon(*challenger)
                                } else {
                                    ChallengeWon(*challenger)
                                });
                        }
                    }
                }
            }
            _ => panic!("Invalid response to given input type"),
        }
    }

    pub fn player_count(&self) -> usize {
        self.player_hands.len()
    }

    fn player(&self) -> usize {
        match self.state {
            Playing { current_player } => current_player,
            Bidding { current_bidder, .. } => current_bidder,
            Challenging { challenger, .. } => challenger,
        }
    }

    fn increment_player(&mut self) {
        let player_count = self.player_count();
        let state = &mut self.state;
        match state {
            Playing { current_player } => *current_player = (*current_player + 1) % player_count,
            Bidding {
                current_bidder,
                passed,
                ..
            } => {
                assert_ne!(
                    passed.as_slice(),
                    &vec![true; player_count],
                    "Infinite loop caused by trying to increment player when all players have passed in the bid",
                );

                *current_bidder = (*current_bidder + 1) % player_count;
                while passed[*current_bidder] {
                    *current_bidder = (*current_bidder + 1) % player_count;
                }
            }
            _ => unreachable!("Increment player should not be called unless playing or bidding"),
        }
    }

    fn cards_played_count(&self) -> usize {
        self.cards_played.iter().map(SmallVec::len).sum()
    }

    fn cards_flipped_count(&self) -> Option<usize> {
        if let State::Challenging { flipped, .. } = &self.state {
            Some(flipped.iter().map(SmallVec::len).sum())
        } else {
            None
        }
    }
}
