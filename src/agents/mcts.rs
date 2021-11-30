use super::Agent;
use crate::env::*;
use rand::seq::IteratorRandom;

#[derive(Debug)]
pub struct MonteAgent {
    tree: UCTNode,
}

impl Agent for MonteAgent {
    fn step(&mut self, _: &GameRequest, _: u64) -> MoveResponse {
        // TODO: Implement

        let mut rng = rand::thread_rng();
        MoveResponse::new(Direction::iter().choose(&mut rng).unwrap_or(Direction::Up))
    }
    fn end(&mut self, _: &GameRequest) {}
}

#[derive(Debug)]
struct UCTNode {}
