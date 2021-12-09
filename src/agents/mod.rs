use std::str::FromStr;
use std::string::ToString;

mod original;
pub use original::*;
mod mobility;
pub use mobility::*;
mod flood;
pub use flood::*;
mod random;
pub use random::*;
pub mod maxn;
pub mod expectimax;

use super::env::{GameRequest, MoveResponse};

const MAX_BOARD_SIZE: usize = 19;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Agent {
    Mobility(MobilityAgent),
    Tree(TreeHeuristic),
    Flood(FloodHeuristic),
    FloodExp(FloodHeuristic),
    Random(RandomAgent),
}

impl Default for Agent {
    fn default() -> Self {
        Self::Mobility(MobilityAgent::default())
    }
}

impl Agent {
    pub async fn step(&self, request: &GameRequest, latency: u64) -> MoveResponse {
        // If the board is very large default to the random agent.
        if request.board.width > MAX_BOARD_SIZE || request.board.height > MAX_BOARD_SIZE {
            return RandomAgent.step(request, latency).await;
        }

        match self {
            Agent::Mobility(agent) => agent.step(request, latency).await,
            Agent::Tree(agent) => maxn::step(agent, request, latency).await,
            Agent::Flood(agent) => maxn::step(agent, request, latency).await,
            Agent::FloodExp(agent) => expectimax::step(agent, request, latency).await,
            Agent::Random(agent) => agent.step(request, latency).await,
        }
    }
}

impl FromStr for Agent {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ToString for Agent {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
