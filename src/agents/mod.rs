use std::sync::{Arc, Mutex};

mod mobility;
pub use mobility::{MobilityAgent, MobilityConfig};
mod random;
pub use random::RandomAgent;
mod tree;
pub use tree::{TreeAgent, TreeConfig};
mod mcts;
pub use mcts::MonteAgent;

use super::env::{GameRequest, MoveResponse};

pub trait Agent: std::fmt::Debug + 'static {
    fn step(&mut self, request: &GameRequest) -> MoveResponse;
    fn end(&mut self, request: &GameRequest);
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Config {
    Mobility(crate::agents::MobilityConfig),
    Tree(crate::agents::TreeConfig),
    Random,
}

impl Default for Config {
    fn default() -> Self {
        Config::Mobility(crate::agents::MobilityConfig::default())
    }
}

impl Config {
    pub fn create_agent(&self, request: &GameRequest) -> Arc<Mutex<dyn Agent + Send>> {
        match self {
            Config::Mobility(config) if request.board.width <= 15 && request.board.height <= 15 => {
                Arc::new(Mutex::new(MobilityAgent::new(request, &config)))
            }
            Config::Tree(config) if request.board.width <= 15 && request.board.height <= 15 => {
                Arc::new(Mutex::new(TreeAgent::new(request, &config)))
            }
            _ => Arc::new(Mutex::new(RandomAgent::default())),
        }
    }
}
