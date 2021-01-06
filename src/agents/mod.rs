mod mobility;
pub use mobility::{MobilityAgent, Config};
mod random;
pub use random::RandomAgent;
mod tree;
pub use tree::TreeAgent;
mod mcts;
pub use mcts::MonteAgent;

use super::env::{GameRequest, MoveResponse};

pub trait Agent: std::fmt::Debug + 'static {
    fn start(&mut self, request: &GameRequest);
    fn step(&mut self, request: &GameRequest) -> MoveResponse;
    fn end(&mut self, request: &GameRequest);
}
