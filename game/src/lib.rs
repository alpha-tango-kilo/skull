use smallvec::{smallvec, SmallVec};

use Card::*;
use State::*;

type Hand = SmallVec<[Card; 4]>;

#[derive(Debug, Copy, Clone)]
pub enum Card {
    Flower,
    Skull,
}

#[derive(Debug)]
pub enum State {
    Playing {
        current_player: u8,
    },
    Bidding {
        current_bidder: u8,
        current_bid: u8,
        max_bid: u8,
        passed: SmallVec<[bool; 6]>,
    },
    Challenging {
        challenger: u8,
        target: u8,
        flipped: SmallVec<[SmallVec<[u8; 4]>; 6]>,
    },
}

impl Default for State {
    fn default() -> Self {
        Playing { current_player: 0 }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InputRequest {
    pub player: u8,
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
    pub fn new(players: u8) -> Self {
        // Range should preferably be checked by wrapper (CLI/GUI)
        assert!((3..=6).contains(&players), "Invalid number of players");

        Game {
            scores: smallvec![0; players as usize],
            player_hands: smallvec![smallvec![Skull, Flower, Flower, Flower]; players as usize],
            cards_played: smallvec![Default::default(); players as usize],
            state: Default::default(),
        }
    }

    pub fn scores(&self) -> &[u8] {
        self.scores.as_slice()
    }

    // TODO: could present as a SmallVec behind a feature gate
    pub fn hands(&self) -> Vec<&[Card]> {
        self.player_hands.iter().map(SmallVec::as_slice).collect()
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn next_input(&self) -> InputRequest {
        use InputType::*;
        let player = self.player();

        match self.state {
            Playing { .. } => {
                if self.cards_played_count() as usize >= self.player_hands.len()
                {
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

    fn player(&self) -> u8 {
        match self.state {
            Playing { current_player } => current_player,
            Bidding { current_bidder, .. } => current_bidder,
            Challenging { challenger, .. } => challenger,
        }
    }

    fn cards_played_count(&self) -> u8 {
        self.cards_played
            .iter()
            // Safe conversion as hand.len() should never be more than 4
            .map(|hand| {
                debug_assert!(hand.len() <= 4, "Hand unexpectedly large");
                hand.len() as u8
            })
            .sum()
    }
}
