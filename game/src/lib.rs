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
enum State {
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
}
