use crate::*;

#[derive(Debug, Clone)]
pub struct Game<const N: usize> {
    scores: [u8; N],                // public via getter
    player_hands: [Hand; N],        // public via getter
    cards_played: [OrderedHand; N], // FVec<[Card; 4]> is ordered bottom -> top
    state: State<N>,                // public via getter
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

    pub const fn scores(&self) -> &[u8] {
        &self.scores
    }

    pub const fn hands(&self) -> &[Hand] {
        &self.player_hands
    }

    pub fn cards_played(&self) -> Vec<&[Card]> {
        self.cards_played.iter().map(FVec::as_slice).collect()
    }

    pub const fn state(&self) -> &State<N> {
        &self.state
    }

    pub fn what_next(&mut self) -> Event {
        use Event::*;
        use InputType::*;
        // if let uses less indendation than match
        if let Some(event) = self.pending_event {
            match event {
                ChallengeStarted => {
                    if let State::Challenging {
                        challenger,
                        target,
                        flipped,
                    } = &mut self.state
                    {
                        // Pull out self.cards_played or else the compiler
                        // will get aggro later
                        let challenger_cards_played =
                            &self.cards_played[*challenger];
                        let challenger_cards_played_count =
                            challenger_cards_played.len();
                        // Flip own cards
                        /*
                        Offset ensures only the correct players cards are
                        flipped, in the event that only some of the
                        player's cards need flipping. If target >
                        challenger_cards_played, then the offset will be
                        0 and all cards will be flipped.
                         */
                        let offset = challenger_cards_played_count
                            .saturating_sub(*target);

                        flipped[*challenger] = (offset
                            ..challenger_cards_played_count)
                            .into_iter()
                            .collect();

                        /*
                        Check if any of those flipped cards are a skull
                        No point in making this a function as we're only
                        going to be doing this one card at a time in future
                        */
                        let flipped_skull =
                            flipped[*challenger].iter().any(|index| {
                                /*println!(
                                    "Card at index {} is a {}",
                                    index, challenger_cards_played[*index]
                                );*/
                                matches!(challenger_cards_played[*index], Skull)
                            });
                        if flipped_skull {
                            self.player_hands[*challenger]
                                .discard_one(&mut self.rng);
                            self.pending_event = Some(ChallengerChoseSkull {
                                challenger: *challenger,
                                skull_player: *challenger,
                            });
                        } else if *target <= challenger_cards_played_count {
                            // If we only need to flip (some of) the
                            // challenger's cards, and have found no skulls,
                            // they've won the challenge
                            self.scores[*challenger] += 1;
                            self.pending_event =
                                if self.scores[*challenger] != 2 {
                                    Some(ChallengeWon(*challenger))
                                } else {
                                    Some(ChallengeWonGameWon(*challenger))
                                };
                        } else {
                            // Nothing exciting has happened, challenger needs
                            // to continue flipping cards
                            self.pending_event = None;
                        }
                    } else {
                        panic!("ChallengeStarted pending event but state isn't Challenging");
                    }
                }
                ChallengerChoseSkull {
                    challenger,
                    skull_player,
                } => {
                    // Transition back to playing
                    self.state = State::Playing {
                        current_player: skull_player,
                    };
                    self.reset_cards_played();
                    if !self.is_player_out(challenger) {
                        self.pending_event = None;
                    } else {
                        // Got themselves out, sad horn (skip them)
                        if challenger == skull_player {
                            self.increment_player();
                        }
                        self.pending_event = Some(PlayerOut(challenger));
                    }
                }
                ChallengeWon(player) => {
                    // Transition back to playing
                    self.state = State::Playing {
                        current_player: player,
                    };
                    self.reset_cards_played();
                    self.pending_event = None;
                }
                Input { .. } => unreachable!(
                    "Input events should never be stored as a pending event"
                ),
                _ => self.pending_event = None, // No-ops: BidStarted, ChallengeWonGameWon, PlayerOut
            }
            event
        } else {
            Event::Input {
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
            }
        }
    }

    pub fn respond(&mut self, response: Response) -> Result<(), ResponseError> {
        use ResponseError::*;
        if self.pending_event.is_some() {
            return Err(PendingEvent);
        }

        // These both have to be worked out before we start working mutably
        // with Game, even though they aren't always used
        let player_count = self.player_count();
        let played_count = self.cards_played_count();

        use Response::*;
        match (&mut self.state, response) {
            // Playing card
            (Playing { current_player }, PlayCard(card)) => {
                /*
                Check player is playing a card they have and haven't already
                played. To do this work out the cards left in their hand, then
                see if the card they're currently trying to play is in that set
                of remaining cards
                 */
                let cards_remaining = (self.player_hands[*current_player]
                    - self.cards_played[*current_player].as_slice())
                .unwrap_or_else(|err| panic!("{}", err));
                if !cards_remaining.has(card) {
                    return Err(CardNotInHand);
                }
                // We're all good, play the card
                self.cards_played[*current_player].push(card).unwrap();
                self.increment_player();
            }
            // Starting bid
            (Playing { current_player }, Bid(n)) => {
                if n > played_count {
                    return Err(BidTooHigh(self.cards_played_count()));
                }

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
                    highest_bid,
                    max_bid,
                    ..
                },
                Bid(n),
            ) => {
                if n > *max_bid {
                    return Err(BidTooHigh(*max_bid));
                } else if n <= *highest_bid {
                    return Err(BidTooLow(*highest_bid + 1));
                } else {
                    // Set the new highest bid(der)
                    *highest_bid = n;
                    *highest_bidder = *current_bidder;

                    // Check if bid is at max and start challenge if so
                    if highest_bid == max_bid {
                        self.pending_event = Some(ChallengeStarted);
                        self.state = Challenging {
                            challenger: *highest_bidder,
                            target: *highest_bid,
                            flipped: [Self::STATE_FLIPPED_INIT; N],
                        }
                    } else {
                        self.increment_player();
                    }
                }
            }
            // Player passes on bid
            (
                Bidding {
                    current_bidder,
                    highest_bidder,
                    highest_bid,
                    passed,
                    ..
                },
                Pass,
            ) => {
                debug_assert!(
                    !passed[*current_bidder],
                    "Current bidder shouldn't have passed, increment player probably went wrong"
                );
                passed[*current_bidder] = true;
                // If all players apart from the highest bidder have passed
                if passed.iter().filter(|b| **b).count() == N - 1 {
                    self.pending_event = Some(ChallengeStarted);
                    self.state = Challenging {
                        challenger: *highest_bidder,
                        target: *highest_bid,
                        flipped: [Self::STATE_FLIPPED_INIT; N],
                    }
                } else {
                    self.increment_player();
                }
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
                if player_index >= player_count
                    || card_index >= self.cards_played[player_index].len()
                {
                    return Err(InvalidIndex);
                } else if player_index == *challenger {
                    return Err(ManuallyFlippingOwnCards);
                } else if flipped[player_index].contains(&card_index) {
                    return Err(CardAlreadyFlipped);
                }

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
                        if len_2d(flipped) == *target {
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
            _ => {
                if let Input { input, .. } = self.what_next() {
                    return Err(IncorrectInputType(input));
                } else {
                    unreachable!("Game must be waiting on an input at there's no pending event");
                }
            }
        }
        Ok(())
    }

    pub const fn player_count(&self) -> usize {
        N
    }

    pub fn remaining_player_count(&self) -> usize {
        self.player_hands.iter().filter(|h| !h.empty()).count()
    }

    const fn player(&self) -> usize {
        match self.state {
            Playing { current_player } => current_player,
            Bidding { current_bidder, .. } => current_bidder,
            Challenging { challenger, .. } => challenger,
        }
    }

    fn set_player(&mut self, player_index: usize) {
        match &mut self.state {
            Playing { current_player } => *current_player = player_index,
            Bidding { current_bidder, .. } => *current_bidder = player_index,
            // This is almost certainly *not* what I want to do, but I've included it
            // for completeness
            Challenging { challenger, .. } => *challenger = player_index,
        }
    }

    // False if not bidding or player hasn't passed
    const fn has_passed(&self, player_index: usize) -> bool {
        if let State::Bidding { passed, .. } = self.state {
            passed[player_index]
        } else {
            false
        }
    }

    fn increment_player(&mut self) {
        // Pre-flight checks
        assert!(
            !self.player_hands.iter().all(|h| h.empty()),
            "All players are out, panicking to avoid infinite loop"
        );
        debug_assert!(
            !matches!(self.state, State::Challenging { .. }),
            "Increment player should never be called when challenging"
        );

        const RANGED_ADDER: fn(usize, usize) -> usize =
            |index, max| (index + 1) % max;
        let mut player_index = RANGED_ADDER(self.player(), self.player_count());
        loop {
            if self.is_player_out(player_index) || self.has_passed(player_index)
            {
                player_index = RANGED_ADDER(player_index, self.player_count());
            } else {
                break;
            }
        }
        self.set_player(player_index);
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

    fn reset_cards_played(&mut self) {
        const EMPTY: OrderedHand = fvec![];
        self.cards_played = [EMPTY; N];
    }

    // Motto: assume nothing, check if game state is valid
    // When I actually hit stable releases, this should only be needed for ensuring
    // Game::create_from isn't being abused. For now though, it'll be used a lot
    fn assert_valid(&self) {
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

        // Ensure <=1 difference in number of cards played per player, ignoring
        // players that are out
        let mut number_of_cards_played = self
            .cards_played
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.is_player_out(*i))
            .map(|(_, fv)| fv.len())
            .collect::<FVec<usize, N>>();
        number_of_cards_played.sort_unstable();
        assert!(
            number_of_cards_played[self.remaining_player_count() - 1]
                - number_of_cards_played[0]
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
                // At most all but two players can have passed
                assert!(
                    passed.iter().filter(|b| **b).count()
                        <= self.remaining_player_count() - 2,
                    "Too many players have passed"
                );
            }
            State::Challenging {
                challenger,
                target,
                flipped,
            } => {
                assert!(
                    !self.is_player_out(*challenger),
                    "Challenger mustn't be out"
                );
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
                        if self.cards_played[*challenger][offset..]
                            .contains(&Skull)
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
        assert!((3..=6).contains(&N), "Invalid number of players");
        let g = Game {
            scores,
            player_hands,
            cards_played,
            state,
            pending_event,
            rng: Default::default(),
        };
        g.assert_valid();
        println!("Game is valid");
        g
    }
}

impl<const N: usize> Default for Game<N> {
    fn default() -> Self {
        Game::new()
    }
}

fn len_2d<T: AsRef<[I]>, I>(arr: &[T]) -> usize {
    arr.iter().map(|sublist| sublist.as_ref().len()).sum()
}

fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + std::hash::Hash,
{
    let mut uniq = std::collections::HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
