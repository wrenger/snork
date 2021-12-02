use crate::env::*;
use rand::seq::IteratorRandom;

#[derive(Debug, Default)]
pub struct RandomAgent;

impl RandomAgent {
    pub async fn step(&mut self, _: &GameRequest, _: u64) -> MoveResponse {
        let mut rng = rand::thread_rng();
        MoveResponse::new(Direction::iter().choose(&mut rng).unwrap_or(Direction::Up))
    }
}
