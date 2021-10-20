use heapless::Vec as FVec; // Fixed Vec

use std::convert::TryFrom;
use std::fmt;

use rand::rngs::ThreadRng;
use rand::Rng;

use Card::*;
use Event::*;
use State::*;

type OrderedHand = FVec<Card, 4>;

macro_rules! fvec {
    () => {
        $crate::FVec::new()
    };
    ($elem:expr; $n:expr) => {
        $crate::FVec::from_slice(&[$elem; $n]).unwrap()
    };
    ( $( $x:expr ),* ) => {
        $crate::FVec::from_slice(&[$($x),*]).unwrap()
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

// TODO: move Game into its own file
#[derive(Debug, Clone)]
pub struct Game<const N: usize> {
    scores: [u8; N],                        // public via getter
    player_hands: [Hand; N],                // public via getter
    cards_played: [OrderedHand; N],       // FVec<[Card; 4]> is ordered bottom -> top
    state: State<N>,                        // public via getter
    pending_event: Option<Event>,
    rng: ThreadRng,
}

impl<const N: usize> Game<N> {
    const CARDS_PLAYED_INIT: OrderedHand = fvec![];
    const STATE_FLIPPED_INIT: FVec<usize, 4> = fvec![];

    pub fn new() -> Self {
        assert!((3..=6).contains(&N), "Invalid number of players");

        Game {
            scores: [0; N],
            player_hands: [Hand::new(); N],
            cards_played: [Self::CARDS_PLAYED_INIT; N],
            state: Playing { current_player: 0 },
            pending_event: None,
            rng: rand::thread_rng(),
        }
    }

    pub fn scores(&self) -> &[u8] {
        &self.scores
    }

    pub fn hands(&self) -> &[Hand] {
        &self.player_hands
    }

    pub fn cards_played(&self) -> Vec<&[Card]> {
        self.cards_played
            .iter()
            .map(FVec::as_slice)
            .collect()
    }

    pub fn state(&self) -> &State<N> {
        &self.state
    }

    pub fn what_next(&mut self) -> Event {
        use Event::*;
        use InputType::*;
        match self.pending_event {
            Some(event) => {
                match event {
                    ChallengeStarted => todo!("Flip own cards / Check for own skulls"),
                    ChallengerChoseSkull { skull_player, .. } => {
                        // Transition back to playing
                        self.state = State::Playing { current_player: skull_player };
                    }
                    ChallengeWon(player) => {
                        // Transition back to playing
                        self.state = State::Playing { current_player: player };
                    }
                    Input { .. } => unreachable!("Input events should never be stored as a pending event"),
                    _ => {} // No-ops: BidStarted, ChallengeWonGameWon
                }
                self.pending_event = None;
                event
            }
            None => Event::Input {
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
            },
        }
    }

    pub fn respond(&mut self, response: Response) {
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
                // TODO: check if playing card they've already played
                assert!(
                    self.player_hands[*current_player].has(card),
                    "Player is playing card they don't have\nHand: {}\nCard: {}",
                    self.player_hands[*current_player],
                    card
                );
                self.cards_played[*current_player].push(card).unwrap();
                self.increment_player();
            }
            // Starting bid
            (Playing { current_player }, Bid(n)) => {
                assert!(
                    n <= played_count,
                    "Started bid for more cards than are in play"
                );
                if n < played_count {
                    self.state = State::Bidding {
                        current_bidder: (*current_player + 1) % player_count,
                        highest_bid: n,
                        highest_bidder: *current_player,
                        max_bid: played_count,
                        passed: [false; N],
                    };
                    self.pending_event = Some(BidStarted);
                } else {
                    // Start bid on max, instantly start challenge
                    self.state = State::Challenging {
                        challenger: *current_player,
                        target: played_count,
                        flipped: [Self::STATE_FLIPPED_INIT; N],
                    };
                    self.pending_event = Some(ChallengeStarted);
                }
            }
            // Raising bid
            (
                Bidding {
                    current_bidder,
                    highest_bidder,
                    highest_bid: current_bid,
                    max_bid,
                    ..
                },
                Bid(n),
            ) => {
                assert!(
                    n <= *max_bid,
                    "Bid greater than maximum ({} > {})",
                    n,
                    max_bid
                );
                assert!(
                    n > *current_bid,
                    "Bid less than current ({} < {})",
                    n,
                    current_bid
                );
                *max_bid = n;
                *highest_bidder = *current_bidder;
                self.increment_player();
                todo!("check if bid is at max and start challenge if so");
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
                    player_index < player_count,
                    "Invalid player specified"
                );
                assert!(
                    card_index
                        < self.player_hands[player_index].count() as usize,
                    "Invalid card specified"
                );
                assert!(
                    !flipped[player_index].contains(&card_index),
                    "Tried to flip already-flipped card"
                );

                let card_flipped = self.cards_played[player_index][card_index];
                use Card::*;
                match card_flipped {
                    Skull => {
                        self.player_hands[*challenger]
                            .discard_one(&mut self.rng);
                        self.pending_event = Some(ChallengerChoseSkull {
                            challenger: *challenger,
                            skull_player: player_index,
                        });
                    }
                    Flower => {
                        flipped[player_index].push(card_index).unwrap();
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

    pub fn remaining_player_count(&self) -> usize {
        self.player_hands.iter().filter(|h| !h.empty()).count()
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
                // TODO: make not use vec!
                assert_ne!(
                    &passed[..],
                    vec![true; player_count].as_slice(),
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

    fn is_player_out(&self, player_index: usize) -> bool {
        self.player_hands
            .get(player_index)
            .expect("Out of range player index")
            .empty()
    }

    fn cards_played_count(&self) -> usize {
        self.cards_played.iter().map(|fv| fv.len()).sum()
    }

    fn cards_flipped_count(&self) -> Option<usize> {
        if let State::Challenging { flipped, .. } = &self.state {
            Some(flipped.iter().map(|fv| fv.len()).sum())
        } else {
            None
        }
    }

    // Motto: assume nothing, check if game state is valid
    // When I actually hit stable releases, this should only be needed for ensuring
    // Game::create_from isn't being abused. For now though, it'll be used a lot
    fn assert_valid(&self) {
        // Ensure numbers of players are consistent across everything except state
        assert!(
            (3..=6).contains(&self.scores.len()),
            "Invalid number of players"
        );
        assert_eq!(
            self.scores.len(),
            self.player_hands.len(),
            "Inconsistent number of players"
        );
        assert_eq!(
            self.scores.len(),
            self.cards_played.len(),
            "Inconsistent number of players"
        );
        assert!(
            !self.scores.iter().any(|s| *s > 2),
            "No one should have a score of more than 2"
        );

        // Ensure hands are valid
        self.player_hands.iter().for_each(|h| h.assert_valid());

        // Ensure cards played are valid
        self.cards_played.iter()
            .enumerate()
            .for_each(|(player_index, ordered_cards)| {
                // Check cards played are legal
                let played_hand = Hand::try_from(ordered_cards.as_slice());
                let played_hand = played_hand.expect("Played cards make invalid hand");
                // Check cards played could have been played from player's hand
                assert!(
                    self.player_hands[player_index].is_superset_of(played_hand),
                    "Player has cards on the table that they shouldn't, based on the cards available to them"
                );
        });
        if !matches!(self.state, Playing { .. }) {
            // Only required if bidding or challenging
            assert!(
                self.cards_played_count() >= self.remaining_player_count(),
                "Less cards played than there are players"
            );
        }

        // Ensure scores is valid
        let players_with_winning_score =
            self.scores.iter().filter(|s| **s == 2).count();
        if let Some(ChallengeWonGameWon(winner_index)) = self.pending_event {
            assert_eq!(
                players_with_winning_score, 1,
                "One player was expected to have a winning score"
            );
            assert!(
                !self.is_player_out(winner_index),
                "Winning player has no cards, meaning they are out"
            );
        } else {
            assert_eq!(
                players_with_winning_score, 0,
                "No players were expected to have a winning score"
            );
        }

        // Ensure <=1 difference in number of cards played per player
        // TODO: check this allows for players to be out
        let mut number_of_cards_played = self
            .cards_played
            .iter()
            .map(|fv| fv.len())
            .collect::<FVec<usize, N>>();
        number_of_cards_played.sort_unstable();
        // TODO(panic): attempt to subtract with overflow
        assert!(
            number_of_cards_played[0]
                - number_of_cards_played[self.player_count() - 1]
                <= 1,
            "Some players have played 2+ more cards than others",
        );

        match &self.state {
            Playing { current_player } => {
                assert!(
                    *current_player < self.scores.len(),
                    "Current player index out of range"
                );
                assert!(
                    !self.is_player_out(*current_player),
                    "Current player mustn't be out"
                );
            }
            Bidding {
                current_bidder,
                highest_bid: current_bid,
                highest_bidder,
                max_bid,
                passed,
            } => {
                assert!(
                    *current_bidder < self.scores.len(),
                    "Current bidder index out of range"
                );
                assert!(
                    !self.is_player_out(*current_bidder),
                    "Current bidder mustn't be out"
                );
                assert!(
                    current_bid < max_bid,
                    "Current bid must be strictly less than maximum (else a challenge should have started"
                );
                assert!(
                    *highest_bidder < self.scores.len(),
                    "Highest bidder out of range"
                );
                assert!(
                    !self.is_player_out(*highest_bidder),
                    "Current bidder mustn't be out"
                );
                assert_ne!(
                    current_bidder, highest_bidder,
                    "Current and highest bidder mustn't be same person"
                );
                // TODO: at most all but two players can have passed
                assert!(
                    !passed.iter().all(|b| *b),
                    "Not all players can have passed"
                );
            }
            State::Challenging {
                challenger,
                target,
                flipped,
            } => {
                // TODO: check if challenger is out (if possible?)
                assert!(
                    *challenger < self.scores.len(),
                    "Challenger index out of range"
                );
                let cards_played = self.cards_played_count();
                assert!(
                    *target <= cards_played,
                    "Target larger than number of cards played"
                );
                assert!(
                    *target >= self.cards_flipped_count().unwrap(),
                    "More cards flipped than targetted"
                );

                // Ensuring flipping is valid
                flipped.iter().zip(self.cards_played.iter()).for_each(
                    |(indexes, played)| {
                        assert!(
                            indexes.len() <= played.len(),
                            "More cards flipped than there are cards"
                        );
                        // Ensure no flipped indexes exceed the number of cards played
                        assert!(
                            !indexes.iter().any(|i| *i >= played.len()),
                            "Out of range index in cards flipped"
                        );
                        // Ensure all flipped indexes have no duplicates
                        assert!(
                            has_unique_elements(indexes),
                            "Duplicate indexes given"
                        );
                    },
                );

                // Ensure correct cards of challenger's are flipped
                // (if the challenge has been announced)
                if !matches!(self.pending_event, Some(ChallengeStarted)) {
                    if *target <= self.cards_played[*challenger].len() {
                        // Flipping subset of own cards
                        let offset = 4 - *target;
                        assert_eq!(
                            // Assume that flipped is sorted for own cards (low - high)
                            flipped[*challenger].as_slice(),
                            // Produces list from offset to one below number of cards
                            // e.g. offset = 1, 4 cards: &[1, 2, 3]
                            (offset..self.player_hands[*challenger].count() as usize).collect::<Vec<_>>().as_slice(),
                            "Challenger hasn't flipped their own cards that they are required to flip"
                        );
                        if self.cards_played[*challenger][offset..].contains(&Skull)
                        {
                            self.assert_self_skull_correctly_declared();
                        }
                    } else {
                        // Flipping all of own cards
                        assert_eq!(
                            flipped[*challenger].len(),
                            self.cards_played[*challenger].len(),
                            "Challenger hasn't flipped all of their own cards when they needed to"
                        );
                        if self.cards_played[*challenger].contains(&Skull) {
                            self.assert_self_skull_correctly_declared();
                        }
                    }
                }

                // Ensure number of flipped skulls is correct
                if let Some(ChallengerChoseSkull { .. }) = self.pending_event {
                    assert_eq!(
                        self.flipped_skulls(),
                        1,
                        "Expected one skull to have been flipped as there's a challenger chose skull pending event"
                    );
                } else {
                    assert_eq!(
                        self.flipped_skulls(),
                        0,
                        "Expected no skulls to have been flipped as there's no pending event"
                    );
                }

                // Ensure there's a pending event if target reached (challenge won)
                if self.cards_flipped_count().unwrap() == *target {
                    if self.scores[*challenger] != 2 {
                        assert_eq!(
                            self.pending_event,
                            Some(ChallengeWon(*challenger)),
                            "Challenge not declared as won or declared as won by incorrect player"
                        );
                    } else {
                        assert_eq!(
                            self.pending_event,
                            Some(ChallengeWonGameWon(*challenger)),
                            "Challenge & game not declared as won or declared as won by incorrect player"
                        );
                    }
                }
            }
        }
    }

    // Only used in Game::assert_valid
    fn flipped_skulls(&self) -> usize {
        if let State::Challenging { flipped, .. } = &self.state {
            flipped
                .iter()
                .zip(self.cards_played.iter())
                .map(|(indexes_flipped, cards_played)| {
                    indexes_flipped
                        .iter()
                        .filter(|i| matches!(cards_played[**i], Skull))
                        .count()
                })
                .sum()
        } else {
            panic!("Requested number of flipped skulls when not challenging");
        }
    }

    // Only call if you know a skull has been turned that was played by the challenger
    fn assert_self_skull_correctly_declared(&self) {
        if let Some(ChallengerChoseSkull {
            challenger,
            skull_player,
        }) = self.pending_event
        {
            assert_eq!(
                challenger, skull_player,
                "Challenger chose own skull but pending event reports differently"
            );
        } else {
            panic!("Challenger chose own skull but event not pending for this");
        }
    }

    pub fn create_from(
        scores: [u8; N],
        player_hands: [Hand; N],
        cards_played: [OrderedHand; N],
        state: State<N>,
        pending_event: Option<Event>,
    ) -> Self {
        let g = Game {
            scores,
            player_hands,
            cards_played,
            state,
            pending_event,
            rng: Default::default(),
        };
        g.assert_valid();
        g
    }
}

fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + std::hash::Hash,
{
    let mut uniq = std::collections::HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
