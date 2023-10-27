use std::cell::RefCell;

use crate::env::*;
use crate::game::Game;
use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RandomAgent;

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy())
}

impl RandomAgent {
    pub async fn step(&self, game: &Game) -> MoveResponse {
        let moves = game.valid_moves(0);
        MoveResponse::new(RNG.with_borrow_mut(|rng| moves.choose(rng).unwrap_or(Direction::Up)))
    }
}
