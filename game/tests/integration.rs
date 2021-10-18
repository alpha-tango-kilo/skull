// TODO

mod playing {
    use game::*;
    use smallvec::smallvec;

    #[test]
    fn play_card() {
        let _game = Game::create_from(
            smallvec![0; 3],
            smallvec![Hand::new(); 3],
            smallvec![Default::default(); 3],
            State::Playing { current_player: 0 },
            None,
        );
    }
}
