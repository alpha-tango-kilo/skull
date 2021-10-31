use game::Card::*;
use game::Response::*;
use game::ResponseError::*;
use game::*;

use std::convert::TryFrom;

use heapless::Vec as FVec;

#[test]
fn pending_event() {
    use game::Event::ChallengeStarted;
    let mut game = Game::create_from(
        [0; 3],
        [Hand::new(); 3],
        [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower; 2]],
        State::Challenging {
            challenger: 0,
            target: 5,
            flipped: [FVec::new(), FVec::new(), FVec::new()],
        },
        Some(ChallengeStarted),
    );
    let err = game.respond(Flip(1, 1)).unwrap_err();
    assert_eq!(err, PendingEvent);
}

#[test]
fn incorrect_input_type() {
    let mut game = Game::create_from(
        [1; 3],
        [Hand::new(); 3],
        [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower; 2]],
        State::Challenging {
            challenger: 0,
            target: 5,
            flipped: [fvec![0, 1], fvec![1, 0], fvec![]],
        },
        None,
    );
    let err = game.respond(Bid(6)).unwrap_err();
    assert_eq!(err, IncorrectInputType(InputType::FlipCard));
}

#[test]
fn card_not_in_hand() {
    let mut game = Game::create_from(
        [0; 3],
        [Hand::new(), Hand::new(), Hand::try_from([Flower]).unwrap()],
        [fvec![Flower], fvec![Flower], fvec![]],
        State::Playing { current_player: 2 },
        None,
    );
    let err = game.respond(PlayCard(Skull)).unwrap_err();
    assert_eq!(err, CardNotInHand);

    let mut game = Game::create_from(
        [0; 3],
        [
            Hand::new(),
            Hand::new(),
            Hand::try_from([Flower, Skull]).unwrap(),
        ],
        [fvec![Flower], fvec![Flower], fvec![Flower]],
        State::Playing { current_player: 2 },
        None,
    );
    let err = game.respond(PlayCard(Flower)).unwrap_err();
    assert_eq!(err, CardNotInHand);
}

#[test]
fn bid_too_low() {
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
    let err = game.respond(Bid(1)).unwrap_err();
    assert_eq!(err, BidTooLow(2));
}

#[test]
fn bid_too_high() {
    let max_bid = 3;
    let mut game = Game::create_from(
        [0; 3],
        [Hand::new(), Hand::new(), Hand::try_from([Flower]).unwrap()],
        [fvec![Flower], fvec![Flower], fvec![Flower]],
        State::Bidding {
            current_bidder: 0,
            highest_bid: 1,
            highest_bidder: 2,
            max_bid,
            passed: [false; 3],
        },
        None,
    );
    let err = game.respond(Bid(4)).unwrap_err();
    assert_eq!(err, BidTooHigh(max_bid));
}

#[test]
fn invalid_index() {
    let mut game = Game::create_from(
        [0; 3],
        [Hand::new(); 3],
        [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower, Skull]],
        State::Challenging {
            challenger: 0,
            target: 6,
            flipped: [fvec![0, 1], fvec![], fvec![]],
        },
        None,
    );
    let err = game.respond(Flip(3, 0)).unwrap_err();
    assert_eq!(err, InvalidIndex);

    let mut game = Game::create_from(
        [0; 3],
        [Hand::new(); 3],
        [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower, Skull]],
        State::Challenging {
            challenger: 0,
            target: 6,
            flipped: [fvec![0, 1], fvec![], fvec![]],
        },
        None,
    );
    let err = game.respond(Flip(1, 3)).unwrap_err();
    assert_eq!(err, InvalidIndex);
}

#[test]
fn card_already_flipped() {
    let mut game = Game::create_from(
        [0; 3],
        [Hand::new(); 3],
        [fvec![Flower; 2], fvec![Flower; 2], fvec![Flower, Skull]],
        State::Challenging {
            challenger: 0,
            target: 5,
            flipped: [fvec![0, 1], fvec![1, 0], fvec![]],
        },
        None,
    );
    let err = game.respond(Flip(1, 0)).unwrap_err();
    assert_eq!(err, CardAlreadyFlipped);
}

#[test]
fn manually_flipping_own_cards() {
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
    let err = game.respond(Flip(challenger, 0)).unwrap_err();
    assert_eq!(err, ManuallyFlippingOwnCards);
}
