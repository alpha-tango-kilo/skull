use smallvec::{smallvec, SmallVec};

use Card::*;
use State::*;

type Hand = SmallVec<[Card; 4]>;

#[derive(Debug, Copy, Clone)]
pub enum Card {
    Flower,
    Skull,
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
    scores: SmallVec<[usize; 6]>,
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
            player_hands: smallvec![smallvec![Skull, Flower, Flower, Flower]; players],
            cards_played: smallvec![Default::default(); players],
            state: Default::default(),
        }
    }

    pub fn scores(&self) -> &[usize] {
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
                if self.cards_played_count() >= self.player_hands.len() {
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

    fn cards_played_count(&self) -> usize {
        self.cards_played.iter().map(SmallVec::len).sum()
    }
}
