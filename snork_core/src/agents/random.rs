use super::Agent;
use crate::env::*;
use rand::seq::IteratorRandom;

#[derive(Debug, Default)]
pub struct RandomAgent;

impl Agent for RandomAgent {
    fn step(&mut self, _: &GameRequest, _: u64) -> MoveResponse {
        let mut rng = rand::thread_rng();
        MoveResponse::new(Direction::iter().choose(&mut rng).unwrap_or(Direction::Up))
    }
    fn end(&mut self, _: &GameRequest) {}
}
