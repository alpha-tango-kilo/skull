mod playing {
    use game::Card::*;
    use game::Event::*;
    use game::Response::*;
    use game::*;

    use std::convert::TryFrom;

    use smallvec::smallvec;

    #[test]
    fn play_card() {
        let mut game = Game::new(3);
        game.respond(PlayCard(Flower));
        assert!(
            matches!(game.state(), &State::Playing { .. }),
            "Game should still be in playing state"
        );

        let mut game = Game::new(3);
        game.respond(PlayCard(Skull));
        assert!(
            matches!(game.state(), &State::Playing { .. }),
            "Game should still be in playing state"
        );
    }

    #[test]
    fn play_card_or_start_bid() {
        let mut game_one = Game::create_from(
            smallvec![0; 3],
            smallvec![Hand::new(); 3],
            smallvec![smallvec![Flower]; 3],
            State::Playing { current_player: 0 },
            None,
        );
        let mut game_two = game_one.clone();

        game_one.respond(PlayCard(Flower));
        assert!(
            matches!(game_one.state(), &State::Playing { .. }),
            "Game should still be in playing state"
        );

        game_two.respond(Bid(2));
        if let State::Bidding {
            current_bidder,
            highest_bid: current_bid,
            highest_bidder,
            max_bid,
            passed,
        } = game_two.state()
        {
            assert_eq!(
                *current_bidder, 1,
                "Current bidder incorrect when bid started (should have incremented)"
            );
            assert_eq!(
                *current_bid, 2,
                "Highest bid set incorrectly when bid started"
            );
            assert_eq!(
                *highest_bidder, 0,
                "Highest bidder set incorrectly when bid started"
            );
            assert_eq!(
                *max_bid, 3,
                "Maximum bid set incorrectly when bid started"
            );
            assert_eq!(
                passed.as_slice(),
                &[false, false, false],
                "Players marked as passed when bid started"
            );
        } else {
            panic!("Game should still be bidding");
        }
        assert_eq!(
            game_two.what_next(),
            BidStarted,
            "BidStarted event not fired"
        );
    }

    #[test]
    fn force_bid() {
        // Player only has one flower which is already in play
        let mut game = Game::create_from(
            smallvec![0; 3],
            smallvec![
                Hand::new(),
                Hand::new(),
                Hand::try_from(&[Flower][..]).unwrap()
            ],
            smallvec![smallvec![Flower]; 3],
            State::Playing { current_player: 2 },
            None,
        );
        game.respond(Bid(2));
        if let State::Bidding {
            current_bidder,
            highest_bid: current_bid,
            highest_bidder,
            max_bid,
            passed,
        } = game.state()
        {
            assert_eq!(
                *current_bidder, 0,
                "Current bidder incorrect when bid started (should have incremented)"
            );
            assert_eq!(
                *current_bid, 2,
                "Highest bid set incorrectly when bid started"
            );
            assert_eq!(
                *highest_bidder, 2,
                "Highest bidder set incorrectly when bid started"
            );
            assert_eq!(
                *max_bid, 3,
                "Maximum bid set incorrectly when bid started"
            );
            assert_eq!(
                passed.as_slice(),
                &[false, false, false],
                "Players marked as passed when bid started"
            );
        } else {
            panic!("Game should still be bidding");
        }
        assert_eq!(game.what_next(), BidStarted, "BidStarted event not fired");
    }
}

mod bidding {
    use game::Card::*;
    use game::Response::*;
    use game::*;

    use std::convert::TryFrom;

    use game::Event::ChallengeStarted;
    use smallvec::smallvec;

    #[test]
    fn bid_no_challenge() {
        let mut game = Game::create_from(
            smallvec![0; 3],
            smallvec![
                Hand::new(),
                Hand::new(),
                Hand::try_from(&[Flower][..]).unwrap()
            ],
            smallvec![smallvec![Flower]; 3],
            State::Bidding {
                current_bidder: 0,
                highest_bid: 1,
                highest_bidder: 2,
                max_bid: 3,
                passed: smallvec![false; 3],
            },
            None,
        );
        game.respond(Bid(2));
    }

    #[test]
    fn bid_starts_challenge() {
        let mut game = Game::create_from(
            smallvec![0; 3],
            smallvec![
                Hand::new(),
                Hand::new(),
                Hand::try_from(&[Flower][..]).unwrap()
            ],
            smallvec![smallvec![Flower]; 3],
            State::Bidding {
                current_bidder: 0,
                highest_bid: 2,
                highest_bidder: 2,
                max_bid: 3,
                passed: smallvec![false; 3],
            },
            None,
        );
        game.respond(Bid(3));
        assert_eq!(
            game.what_next(),
            ChallengeStarted,
            "ChallengeStarted event not fired"
        );
    }

    #[test]
    fn pass_no_challenge() {
        let bidder = 0;
        let mut game = Game::create_from(
            smallvec![0; 3],
            smallvec![
                Hand::new(),
                Hand::new(),
                Hand::try_from(&[Flower][..]).unwrap()
            ],
            smallvec![smallvec![Flower]; 3],
            State::Bidding {
                current_bidder: bidder,
                highest_bid: 2,
                highest_bidder: 2,
                max_bid: 3,
                passed: smallvec![false; 3],
            },
            None,
        );
        game.respond(Pass);

        if let State::Bidding { passed, .. } = game.state() {
            assert!(
                passed[bidder],
                "Bidder {} should have been marked as passed",
                bidder
            );
        } else {
            panic!("Game should still be bidding");
        }
    }

    #[test]
    fn pass_starts_challenge() {
        let bidder = 0; // Changing will break test
        let mut game = Game::create_from(
            smallvec![0; 3],
            smallvec![
                Hand::new(),
                Hand::new(),
                Hand::try_from(&[Flower][..]).unwrap()
            ],
            smallvec![smallvec![Flower]; 3],
            State::Bidding {
                current_bidder: bidder,
                highest_bid: 2,
                highest_bidder: 2,
                max_bid: 3,
                passed: smallvec![false, true, false],
            },
            None,
        );
        game.respond(Pass);

        if let State::Challenging {
            challenger, target, ..
        } = game.state()
        {
            assert_eq!(*challenger, 2, "Incorrect challenger chosen");
            assert_eq!(*target, 2, "Incorrect target for challenge");
        } else {
            panic!("Game state should have changed to challenge");
        }

        assert_eq!(
            game.what_next(),
            ChallengeStarted,
            "ChallengeStarted event not fired"
        );
    }
}

mod challenging {
    // TODO: players getting out, flipping more than their own cards
    mod flipping_own_cards {
        use game::Card::*;
        use game::Event::*;
        use game::*;

        use smallvec::smallvec;
        
        #[test]
        fn all_not_win_or_loss() {
            let challenger = 0;
            let mut game = Game::create_from(
                smallvec![0; 3],
                smallvec![Hand::new(); 3],
                smallvec![smallvec![Flower; 2]; 3],
                State::Challenging {
                    challenger,
                    target: 5,
                    flipped: smallvec![smallvec![]; 3],
                },
                Some(ChallengeStarted),
            );
            assert_eq!(
                game.what_next(),
                ChallengeStarted,
                "ChallengeStarted event not emitted (despite being provided)"
            );
            // Check challenger's own cards have been automatically flipped
            if let State::Challenging { flipped, .. } = game.state() {
                assert_eq!(
                    flipped[challenger].as_slice(),
                    &[0, 1],
                    "Challenger's cards not correctly flipped"
                );
            }

            assert_eq!(
                game.what_next(),
                Input {
                    player: challenger,
                    input: InputType::FlipCard,
                },
                "Challenger not prompted for further input"
            );
        }

        #[test]
        fn all_loss() {
            let challenger = 0;
            let mut game = Game::create_from(
                smallvec![0; 3],
                smallvec![Hand::new(); 3],
                smallvec![smallvec![Flower, Skull]; 3],
                State::Challenging {
                    challenger,
                    target: 5,
                    flipped: smallvec![smallvec![]; 3],
                },
                Some(ChallengeStarted),
            );
            assert_eq!(
                game.what_next(),
                ChallengeStarted,
                "ChallengeStarted event not emitted (despite being provided)"
            );
            // Check challenger's own cards have been automatically flipped
            if let State::Challenging { flipped, .. } = game.state() {
                assert_eq!(
                    flipped[challenger].as_slice(),
                    &[0, 1],
                    "Challenger's cards not correctly flipped"
                );
            }

            assert_eq!(
                game.what_next(),
                ChallengerChoseSkull {
                    challenger,
                    skull_player: challenger,
                },
                "ChallengerChoseSkull event not fired"
            );
            assert_eq!(
                game.what_next(),
                Input {
                    player: challenger,
                    input: InputType::PlayCard,
                },
                "Playing didn't resume after lost challenge (or didn't resume from correct player)"
            );
            assert_eq!(
                game.hands()[challenger].count(),
                3,
                "Losing challenger didn't have card discarded"
            );
        }

        #[test]
        fn all_win() {
            let challenger = 0;
            let mut game = Game::create_from(
                smallvec![0; 3],
                smallvec![Hand::new(); 3],
                smallvec![smallvec![Flower; 2]; 3],
                State::Challenging {
                    challenger,
                    target: 2,
                    flipped: smallvec![smallvec![]; 3],
                },
                Some(ChallengeStarted),
            );
            assert_eq!(
                game.what_next(),
                ChallengeStarted,
                "ChallengeStarted event not emitted (despite being provided)"
            );
            // Check challenger's own cards have been automatically flipped
            if let State::Challenging { flipped, .. } = game.state() {
                assert_eq!(
                    flipped[challenger].as_slice(),
                    &[0, 1],
                    "Challenger's cards not correctly flipped"
                );
            }

            assert_eq!(
                game.what_next(),
                ChallengeWon(challenger),
                "ChallengeWon({}) event not emitted",
                challenger
            );
            assert_eq!(
                game.scores()[challenger],
                1,
                "Challenger not awarded one point"
            );
            assert_eq!(
                game.cards_played().as_slice(),
                &[&[], &[], &[]],
                "Cards played was not reset"
            );
            assert_eq!(
                game.what_next(),
                Input {
                    player: challenger,
                    input: InputType::PlayCard,
                },
                "Challenge winner not prompted to play card after ChallengeWon event cleared"
            );
        }

        #[test]
        fn some_win() {
            let challenger = 0;
            let mut game = Game::create_from(
                smallvec![0; 3],
                smallvec![Hand::new(); 3],
                smallvec![smallvec![Skull, Flower]; 3],
                State::Challenging {
                    challenger,
                    target: 1,
                    flipped: smallvec![smallvec![]; 3],
                },
                Some(ChallengeStarted),
            );
            assert_eq!(
                game.what_next(),
                ChallengeStarted,
                "ChallengeStarted event not emitted (despite being provided)"
            );
            // Check challenger's own cards have been automatically flipped
            if let State::Challenging { flipped, .. } = game.state() {
                assert_eq!(
                    flipped[challenger].as_slice(),
                    &[1],
                    "Challenger's cards not correctly flipped"
                );
            }

            assert_eq!(
                game.what_next(),
                ChallengeWon(challenger),
                "ChallengeWon({}) event not emitted",
                challenger
            );
            assert_eq!(
                game.scores()[challenger],
                1,
                "Challenger not awarded one point"
            );
            assert_eq!(
                game.cards_played().as_slice(),
                &[&[], &[], &[]],
                "Cards played was not reset"
            );
            assert_eq!(
                game.what_next(),
                Input {
                    player: challenger,
                    input: InputType::PlayCard,
                },
                "Challenge winner not prompted to play card after ChallengeWon event cleared"
            );
        }

        #[test]
        fn some_loss() {
            let challenger = 0;
            let mut game = Game::create_from(
                smallvec![0; 3],
                smallvec![Hand::new(); 3],
                smallvec![smallvec![Flower, Skull]; 3],
                State::Challenging {
                    challenger,
                    target: 1,
                    flipped: smallvec![smallvec![]; 3],
                },
                Some(ChallengeStarted),
            );
            assert_eq!(
                game.what_next(),
                ChallengeStarted,
                "ChallengeStarted event not emitted (despite being provided)"
            );
            // Check challenger's own cards have been automatically flipped
            if let State::Challenging { flipped, .. } = game.state() {
                assert_eq!(
                    flipped[challenger].as_slice(),
                    &[1],
                    "Challenger's cards not correctly flipped"
                );
            }

            assert_eq!(
                game.what_next(),
                ChallengerChoseSkull {
                    challenger,
                    skull_player: challenger,
                },
                "ChallengerChoseSkull event not fired"
            );
            assert_eq!(
                game.what_next(),
                Input {
                    player: challenger,
                    input: InputType::PlayCard,
                },
                "Playing didn't resume after lost challenge (or didn't resume from correct player)"
            );
            assert_eq!(
                game.hands()[challenger].count(),
                3,
                "Losing challenger didn't have card discarded"
            );
        }
    }
}
