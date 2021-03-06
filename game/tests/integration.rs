mod playing {
    use game::Card::*;
    use game::Event::*;
    use game::Response::*;
    use game::*;

    use std::convert::TryFrom;

    #[test]
    fn play_card() {
        let mut game: Game<3> = Game::new();
        game.respond(PlayCard(Flower))
            .expect("Game should have accepted the response");
        assert!(
            matches!(game.state(), &State::Playing { .. }),
            "Game should still be in playing state"
        );

        let mut game: Game<3> = Game::new();
        game.respond(PlayCard(Skull))
            .expect("Game should have accepted the response");
        assert!(
            matches!(game.state(), &State::Playing { .. }),
            "Game should still be in playing state"
        );
    }

    #[test]
    fn play_card_or_start_bid() {
        let mut game_one = Game::create_from(
            [0; 3],
            [Hand::new(); 3],
            [fvec![Flower], fvec![Flower], fvec![Flower]],
            State::Playing { current_player: 0 },
            None,
        );
        let mut game_two = game_one.clone();

        game_one
            .respond(PlayCard(Flower))
            .expect("Game should have accepted the response");
        assert!(
            matches!(game_one.state(), &State::Playing { .. }),
            "Game should still be in playing state"
        );

        game_two
            .respond(Bid(2))
            .expect("Game should have accepted the response");
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
                passed,
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
            [0; 3],
            [Hand::new(), Hand::new(), Hand::try_from([Flower]).unwrap()],
            [fvec![Flower], fvec![Flower], fvec![Flower]],
            State::Playing { current_player: 2 },
            None,
        );
        game.respond(Bid(2))
            .expect("Game should have accepted the response");
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
                passed,
                &[false, false, false],
                "Players marked as passed when bid started"
            );
        } else {
            panic!("Game should still be bidding");
        }
        assert_eq!(game.what_next(), BidStarted, "BidStarted event not fired");
    }

    #[test]
    fn out_player_skipped() {
        println!("Middle of list");
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(), Hand::default(), Hand::new()],
            [fvec![Flower], fvec![], fvec![Flower]],
            State::Playing { current_player: 0 },
            None,
        );
        game.respond(Response::PlayCard(Skull))
            .expect("Game should have accepted the response");
        if let State::Playing { current_player } = game.state() {
            assert_ne!(
                *current_player,
                1,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_player,
                2,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 1,
                input: InputType::PlayCardOrStartBid,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 2,
                input: InputType::PlayCardOrStartBid,
            },
            "Current player is incorrect (but not a player with no cards) (input request)"
        );

        println!("End of list");
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(), Hand::new(), Hand::default()],
            [fvec![Flower], fvec![Flower], fvec![]],
            State::Playing { current_player: 1 },
            None,
        );
        game.respond(Response::PlayCard(Skull))
            .expect("Game should have accepted the response");
        if let State::Playing { current_player } = game.state() {
            assert_ne!(
                *current_player,
                2,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_player,
                0,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 2,
                input: InputType::PlayCardOrStartBid,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 0,
                input: InputType::PlayCardOrStartBid,
            },
            "Current player is incorrect (but not a player with no cards) (input request)"
        );

        println!("Start of list");
        let mut game = Game::create_from(
            [0; 3],
            [Hand::default(), Hand::new(), Hand::new()],
            [fvec![], fvec![Flower], fvec![Flower]],
            State::Playing { current_player: 2 },
            None,
        );
        game.respond(Response::PlayCard(Skull))
            .expect("Game should have accepted the response");
        if let State::Playing { current_player } = game.state() {
            assert_ne!(
                *current_player,
                0,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_player,
                1,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 0,
                input: InputType::PlayCardOrStartBid,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 1,
                input: InputType::PlayCardOrStartBid,
            },
            "Current player is incorrect (but not a player with no cards) (input request)"
        );
    }

    #[test]
    fn out_players_skipped() {
        println!("In middle");
        let mut game = Game::create_from(
            [0; 4],
            [Hand::new(), Hand::default(), Hand::default(), Hand::new()],
            [fvec![Flower], fvec![], fvec![], fvec![Flower]],
            State::Playing { current_player: 0 },
            None,
        );
        game.respond(Response::PlayCard(Skull))
            .expect("Game should have accepted the response");
        if let State::Playing { current_player } = game.state() {
            assert_ne!(
                *current_player,
                1,
                "Current player is out and should have been skipped (game state)"
            );
            assert_ne!(
                *current_player,
                2,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_player,
                3,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 1,
                input: InputType::PlayCard,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_ne!(
            game.what_next(),
            Input {
                player: 2,
                input: InputType::PlayCard,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 3,
                input: InputType::PlayCard,
            },
            "Current player is incorrect (but not a player with no cards) (input request)"
        );

        println!("At edges");
        let mut game = Game::create_from(
            [0; 4],
            [Hand::default(), Hand::new(), Hand::new(), Hand::default()],
            [fvec![], fvec![Flower], fvec![Flower], fvec![]],
            State::Playing { current_player: 2 },
            None,
        );
        game.respond(Response::PlayCard(Skull))
            .expect("Game should have accepted the response");
        if let State::Playing { current_player } = game.state() {
            assert_ne!(
                *current_player,
                3,
                "Current player is out and should have been skipped (game state)"
            );
            assert_ne!(
                *current_player,
                0,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_player,
                1,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 3,
                input: InputType::PlayCard,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_ne!(
            game.what_next(),
            Input {
                player: 0,
                input: InputType::PlayCard,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 1,
                input: InputType::PlayCard,
            },
            "Current player is incorrect (but not a player with no cards) (input request)"
        );
    }
}

mod bidding {
    use game::Card::*;
    use game::Event::*;
    use game::Response::*;
    use game::*;

    use std::convert::TryFrom;

    #[test]
    fn bid_no_challenge() {
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(), Hand::new(), Hand::try_from([Flower]).unwrap()],
            [fvec![Flower], fvec![Flower], fvec![Flower]],
            State::Bidding {
                current_bidder: 0,
                highest_bid: 1,
                highest_bidder: 2,
                max_bid: 3,
                passed: [false; 3],
            },
            None,
        );
        game.respond(Bid(2))
            .expect("Game should have accepted the response");
    }

    #[test]
    fn bid_starts_challenge() {
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(), Hand::new(), Hand::try_from([Flower]).unwrap()],
            [fvec![Flower], fvec![Flower], fvec![Flower]],
            State::Bidding {
                current_bidder: 0,
                highest_bid: 2,
                highest_bidder: 2,
                max_bid: 3,
                passed: [false; 3],
            },
            None,
        );
        game.respond(Bid(3))
            .expect("Game should have accepted the response");
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
            [0; 3],
            [Hand::new(), Hand::new(), Hand::try_from([Flower]).unwrap()],
            [fvec![Flower], fvec![Flower], fvec![Flower]],
            State::Bidding {
                current_bidder: bidder,
                highest_bid: 2,
                highest_bidder: 2,
                max_bid: 3,
                passed: [false; 3],
            },
            None,
        );
        game.respond(Pass)
            .expect("Game should have accepted the response");

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
            [0; 3],
            [Hand::new(), Hand::new(), Hand::try_from([Flower]).unwrap()],
            [fvec![Flower], fvec![Flower], fvec![Flower]],
            State::Bidding {
                current_bidder: bidder,
                highest_bid: 2,
                highest_bidder: 2,
                max_bid: 3,
                passed: [false, true, false],
            },
            None,
        );
        game.respond(Pass)
            .expect("Game should have accepted the response");

        assert_eq!(
            game.what_next(),
            ChallengeStarted,
            "ChallengeStarted event not fired"
        );

        if let State::Challenging {
            challenger, target, ..
        } = game.state()
        {
            assert_eq!(*challenger, 2, "Incorrect challenger chosen");
            assert_eq!(*target, 2, "Incorrect target for challenge");
        } else {
            panic!("Game state should have changed to challenge");
        }
    }

    #[test]
    fn out_player_skipped() {
        println!("Middle of list");
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(), Hand::default(), Hand::new()],
            [
                fvec![Flower, Flower, Skull],
                fvec![],
                fvec![Flower, Flower, Flower],
            ],
            State::Bidding {
                current_bidder: 0,
                highest_bid: 2,
                highest_bidder: 2,
                max_bid: 6,
                passed: [false; 3],
            },
            None,
        );
        game.respond(Response::Bid(3))
            .expect("Game should have accepted the response");
        if let State::Bidding { current_bidder, .. } = game.state() {
            assert_ne!(
                *current_bidder,
                1,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_bidder,
                2,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 1,
                input: InputType::BidOrPass,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 2,
                input: InputType::BidOrPass,
            },
            "Current player is incorrect (but not a player with no cards) (input request)",
        );

        println!("End of list");
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(), Hand::new(), Hand::default()],
            [
                fvec![Flower, Flower, Skull],
                fvec![Flower, Flower, Flower],
                fvec![],
            ],
            State::Bidding {
                current_bidder: 1,
                highest_bid: 2,
                highest_bidder: 0,
                max_bid: 6,
                passed: [false; 3],
            },
            None,
        );
        game.respond(Response::Bid(3))
            .expect("Game should have accepted the response");
        if let State::Bidding { current_bidder, .. } = game.state() {
            assert_ne!(
                *current_bidder,
                2,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_bidder,
                0,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 2,
                input: InputType::BidOrPass,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 0,
                input: InputType::BidOrPass,
            },
            "Current player is incorrect (but not a player with no cards) (input request)",
        );

        println!("Start of list");
        let mut game = Game::create_from(
            [0; 3],
            [Hand::default(), Hand::new(), Hand::new()],
            [
                fvec![],
                fvec![Flower, Flower, Skull],
                fvec![Flower, Flower, Flower],
            ],
            State::Bidding {
                current_bidder: 2,
                highest_bid: 2,
                highest_bidder: 1,
                max_bid: 6,
                passed: [false; 3],
            },
            None,
        );
        game.respond(Response::Bid(3))
            .expect("Game should have accepted the response");
        if let State::Bidding { current_bidder, .. } = game.state() {
            assert_ne!(
                *current_bidder,
                0,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_bidder,
                1,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 0,
                input: InputType::BidOrPass,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 1,
                input: InputType::BidOrPass,
            },
            "Current player is incorrect (but not a player with no cards) (input request)",
        );
    }

    #[test]
    fn out_players_skipped() {
        println!("Middle of list");
        let mut game = Game::create_from(
            [0; 4],
            [Hand::new(), Hand::default(), Hand::default(), Hand::new()],
            [
                fvec![Flower, Flower, Skull],
                fvec![],
                fvec![],
                fvec![Flower, Flower, Flower],
            ],
            State::Bidding {
                current_bidder: 0,
                highest_bid: 2,
                highest_bidder: 3,
                max_bid: 6,
                passed: [false; 4],
            },
            None,
        );
        game.respond(Response::Bid(3))
            .expect("Game should have accepted the response");
        if let State::Bidding { current_bidder, .. } = game.state() {
            assert_ne!(
                *current_bidder,
                1,
                "Current player is out and should have been skipped (game state)"
            );
            assert_ne!(
                *current_bidder,
                2,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_bidder,
                3,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 1,
                input: InputType::BidOrPass,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_ne!(
            game.what_next(),
            Input {
                player: 2,
                input: InputType::BidOrPass,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 3,
                input: InputType::BidOrPass,
            },
            "Current player is incorrect (but not a player with no cards) (input request)"
        );

        println!("Ends of list");
        let mut game = Game::create_from(
            [0; 4],
            [Hand::default(), Hand::new(), Hand::new(), Hand::default()],
            [
                fvec![],
                fvec![Flower, Flower, Skull],
                fvec![Flower, Flower, Flower],
                fvec![],
            ],
            State::Bidding {
                current_bidder: 2,
                highest_bid: 2,
                highest_bidder: 1,
                max_bid: 6,
                passed: [false; 4],
            },
            None,
        );
        game.respond(Response::Bid(3))
            .expect("Game should have accepted the response");
        if let State::Bidding { current_bidder, .. } = game.state() {
            assert_ne!(
                *current_bidder,
                3,
                "Current player is out and should have been skipped (game state)"
            );
            assert_ne!(
                *current_bidder,
                0,
                "Current player is out and should have been skipped (game state)"
            );
            assert_eq!(
                *current_bidder,
                1,
                "Current player is incorrect (but not a player with no cards) (game state)"
            );
        } else {
            panic!("Game state changed for no reason");
        }
        assert_ne!(
            game.what_next(),
            Input {
                player: 3,
                input: InputType::BidOrPass,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_ne!(
            game.what_next(),
            Input {
                player: 0,
                input: InputType::BidOrPass,
            },
            "Current player is out and should have been skipped (input request)"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 1,
                input: InputType::BidOrPass,
            },
            "Current player is incorrect (but not a player with no cards) (input request)"
        );
    }
}

mod challenging {
    use game::Card::*;
    use game::Event::*;
    use game::*;

    use heapless::Vec as FVec;
    use std::convert::TryFrom;

    #[test]
    fn flipping_other_players_cards() {
        let challenger = 0;
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(); 3],
            [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower, Skull]],
            State::Challenging {
                challenger,
                target: 6,
                flipped: [fvec![0, 1], fvec![], fvec![]],
            },
            None,
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: challenger,
                input: InputType::FlipCard,
            },
            "Expected game to want challenger to flip a card (0)"
        );

        game.respond(Response::Flip(1, 1))
            .expect("Game should have accepted the response");
        if let State::Challenging { flipped, .. } = game.state() {
            let expected: &[FVec<usize, 4>; 3] =
                &[fvec![0, 1], fvec![1], fvec![]];
            assert_eq!(
                flipped, expected,
                "Incorrect card marked as flipped (1)"
            );
        } else {
            panic!("Game state not challenging when it was given no reason to change (1)");
        }
        assert_eq!(
            game.what_next(),
            Input {
                player: challenger,
                input: InputType::FlipCard,
            },
            "Expected game to want challenger to flip a card (1)"
        );

        game.respond(Response::Flip(1, 0))
            .expect("Game should have accepted the response");
        if let State::Challenging { flipped, .. } = game.state() {
            let expected: &[FVec<usize, 4>; 3] =
                &[fvec![0, 1], fvec![1, 0], fvec![]];
            assert_eq!(
                flipped, expected,
                "Incorrect card marked as flipped (2)"
            );
        } else {
            panic!("Game state not challenging when it was given no reason to change (2)");
        }
        assert_eq!(
            game.what_next(),
            Input {
                player: challenger,
                input: InputType::FlipCard,
            },
            "Expected game to want challenger to flip a card (2)"
        );

        game.respond(Response::Flip(2, 0))
            .expect("Game should have accepted the response");
        if let State::Challenging { flipped, .. } = game.state() {
            let expected: &[FVec<usize, 4>; 3] =
                &[fvec![0, 1], fvec![1, 0], fvec![0]];
            assert_eq!(
                flipped, expected,
                "Incorrect card marked as flipped (3)"
            );
        } else {
            panic!("Game state not challenging when it was given no reason to change (3)");
        }
        assert_eq!(
            game.what_next(),
            Input {
                player: challenger,
                input: InputType::FlipCard,
            },
            "Expected game to want challenger to flip a card (3)"
        );
    }

    #[test]
    fn challenge_lost() {
        let challenger = 0;
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(); 3],
            [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower, Skull]],
            State::Challenging {
                challenger,
                target: 5,
                flipped: [fvec![0, 1], fvec![1, 0], fvec![]],
            },
            None,
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: challenger,
                input: InputType::FlipCard,
            },
            "Expected game to want challenger to flip a card"
        );
        game.respond(Response::Flip(2, 1))
            .expect("Game should have accepted the response");
        assert_eq!(
            game.what_next(),
            ChallengerChoseSkull {
                challenger,
                skull_player: 2,
            },
            "ChallengerChoseSkull event not fired",
        );
        assert_eq!(
            game.hands()[challenger].count(),
            3,
            "Challenger didn't have a card discarded"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: 2,
                input: InputType::PlayCard,
            },
            "Playing didn't resume after lost challenge (or didn't resume from correct player)"
        );
        assert_eq!(
            game.cards_played(),
            vec![&[], &[], &[]],
            "Cards played didn't reset"
        );
    }

    #[test]
    fn challenge_lost_player_out() {
        let challenger = 2;
        let skull_player = 0;
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(), Hand::new(), Hand::try_from([Flower]).unwrap()],
            [fvec![Skull], fvec![Flower], fvec![Flower]],
            State::Challenging {
                challenger,
                target: 2,
                flipped: [fvec![], fvec![], fvec![0]],
            },
            None,
        );
        game.respond(Response::Flip(0, 0))
            .expect("Game should have accepted the response");
        assert_eq!(
            game.what_next(),
            ChallengerChoseSkull {
                challenger,
                skull_player,
            },
            "ChallengerChoseSkull event not fired",
        );
        assert_eq!(
            game.hands()[challenger].count(),
            0,
            "Challenger didn't have a card discarded"
        );
        assert_eq!(
            game.what_next(),
            PlayerOut(challenger),
            "{:?} event not fired",
            PlayerOut(challenger),
        );
        assert_eq!(
            game.cards_played(),
            vec![&[], &[], &[]],
            "Cards played didn't reset"
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: skull_player,
                input: InputType::PlayCard,
            },
            "Playing didn't resume after lost challenge (or didn't resume from correct player)"
        );
    }

    #[test]
    fn challenge_won() {
        let challenger = 0;
        let mut game = Game::create_from(
            [0; 3],
            [Hand::new(); 3],
            [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower; 2]],
            State::Challenging {
                challenger,
                target: 5,
                flipped: [fvec![0, 1], fvec![1, 0], fvec![]],
            },
            None,
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: challenger,
                input: InputType::FlipCard,
            },
            "Expected game to want challenger to flip a card"
        );
        game.respond(Response::Flip(2, 1))
            .expect("Game should have accepted the response");
        assert_eq!(
            game.what_next(),
            ChallengeWon(challenger),
            "ChallengeWon({}) event not fired",
            challenger,
        );
        assert_eq!(
            game.scores()[challenger],
            1,
            "Challenger not awarded one point"
        );
    }

    #[test]
    fn challenge_won_game_won() {
        let challenger = 0;
        let mut game = Game::create_from(
            [1; 3],
            [Hand::new(); 3],
            [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower; 2]],
            State::Challenging {
                challenger,
                target: 5,
                flipped: [fvec![0, 1], fvec![1, 0], fvec![]],
            },
            None,
        );
        assert_eq!(
            game.what_next(),
            Input {
                player: challenger,
                input: InputType::FlipCard,
            },
            "Expected game to want challenger to flip a card"
        );
        game.respond(Response::Flip(2, 1))
            .expect("Game should have accepted the response");
        assert_eq!(
            game.what_next(),
            ChallengeWonGameWon(challenger),
            "ChallengeWonGameWon({}) event not fired",
            challenger,
        );
        assert_eq!(
            game.scores()[challenger],
            2,
            "Challenger not awarded one point"
        );
    }

    mod flipping_own_cards {
        use game::Card::*;
        use game::Event::*;
        use game::*;

        use std::convert::TryFrom;

        #[test]
        fn all_not_win_or_loss() {
            let challenger = 0;
            let mut game = Game::create_from(
                [0; 3],
                [Hand::new(); 3],
                [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower; 2]],
                State::Challenging {
                    challenger,
                    target: 5,
                    flipped: [FVec::new(), FVec::new(), FVec::new()],
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
                [0; 3],
                [Hand::new(); 3],
                [
                    fvec![Flower, Skull],
                    fvec![Flower, Skull],
                    fvec![Flower, Skull],
                ],
                State::Challenging {
                    challenger,
                    target: 5,
                    flipped: [fvec![], fvec![], fvec![]],
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
            assert_eq!(
                game.cards_played(),
                vec![&[], &[], &[]],
                "Cards played didn't reset"
            );
        }

        #[test]
        fn all_win() {
            let challenger = 0;
            let mut game = Game::create_from(
                [0; 3],
                [Hand::new(); 3],
                [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower; 2]],
                State::Challenging {
                    challenger,
                    target: 2,
                    flipped: [fvec![], fvec![], fvec![]],
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
        fn all_win_game() {
            let challenger = 0;
            let mut game = Game::create_from(
                [1; 3],
                [Hand::new(); 3],
                [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower; 2]],
                State::Challenging {
                    challenger,
                    target: 2,
                    flipped: [fvec![], fvec![], fvec![]],
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
                ChallengeWonGameWon(challenger),
                "ChallengeWonGameWon({}) event not emitted",
                challenger
            );
            assert_eq!(
                game.scores()[challenger],
                2,
                "Challenger not awarded one point"
            );
        }

        #[test]
        fn some_loss() {
            let challenger = 0;
            let mut game = Game::create_from(
                [0; 3],
                [Hand::new(); 3],
                [
                    fvec![Flower, Skull],
                    fvec![Skull, Flower],
                    fvec![Skull, Flower],
                ],
                State::Challenging {
                    challenger,
                    target: 1,
                    flipped: [fvec![], fvec![], fvec![]],
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
            assert_eq!(
                game.cards_played(),
                vec![&[], &[], &[]],
                "Cards played didn't reset"
            );
        }

        #[test]
        fn some_win() {
            let challenger = 0;
            let mut game = Game::create_from(
                [0; 3],
                [Hand::new(); 3],
                [
                    fvec![Skull, Flower],
                    fvec![Skull, Flower],
                    fvec![Skull, Flower],
                ],
                State::Challenging {
                    challenger,
                    target: 1,
                    flipped: [fvec![], fvec![], fvec![]],
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
        fn some_win_game() {
            let challenger = 0;
            let mut game = Game::create_from(
                [1; 3],
                [Hand::new(); 3],
                [
                    fvec![Skull, Flower],
                    fvec![Skull, Flower],
                    fvec![Skull, Flower],
                ],
                State::Challenging {
                    challenger,
                    target: 1,
                    flipped: [fvec![], fvec![], fvec![]],
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
                ChallengeWonGameWon(challenger),
                "ChallengeWonGameWon({}) event not emitted",
                challenger
            );
            assert_eq!(
                game.scores()[challenger],
                2,
                "Challenger not awarded one point"
            );
        }

        #[test]
        fn player_out() {
            let challenger = 0;
            let mut game = Game::create_from(
                [1; 3],
                [Hand::try_from([Skull]).unwrap(), Hand::new(), Hand::new()],
                [fvec![Skull], fvec![Flower, Skull], fvec![Flower, Skull]],
                State::Challenging {
                    challenger,
                    target: 3,
                    flipped: [fvec![], fvec![], fvec![]],
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
                    &[0],
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
                game.cards_played(),
                vec![&[], &[], &[]],
                "Cards played didn't reset"
            );
            assert!(
                game.hands()[challenger].empty(),
                "Not all of the challenger's cards have been discarded"
            );
            assert_eq!(
                game.what_next(),
                PlayerOut(challenger),
                "PlayerOut event not fired"
            );
            assert_ne!(
                game.what_next(),
                Input {
                    player: challenger,
                    input: InputType::PlayCard,
                },
                "Playing resumed from challenger, who is out"
            );
            assert_eq!(
                game.what_next(),
                Input {
                    player: 1,
                    input: InputType::PlayCard,
                },
                "Playing didn't resume after lost challenge (or didn't resume from correct player)"
            );
        }
    }
}
