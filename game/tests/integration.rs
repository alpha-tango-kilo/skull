// TODO

mod playing {
    use game::Card::*;
    use game::Response::*;
    use game::*;

    use std::convert::TryFrom;

    use smallvec::smallvec;

    #[test]
    fn play_card() {
        let mut game = Game::new(3);
        game.respond(PlayCard(Flower));

        let mut game = Game::new(3);
        game.respond(PlayCard(Skull));
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
        game_two.respond(Bid(2));
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
    }
}
