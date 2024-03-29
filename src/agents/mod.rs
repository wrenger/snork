use std::str::FromStr;
use std::string::ToString;
use std::sync::Arc;

mod original;
pub use original::*;
mod mobility;
pub use mobility::*;
mod flood;
pub use flood::*;
mod random;
pub use random::*;
pub mod maxn;
mod solo;
pub use solo::*;
mod mcts;
pub use mcts::*;

use crate::game::Game;

use super::env::{GameRequest, MoveResponse};

const MAX_BOARD_SIZE: usize = 19;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Agent {
    Mobility(MobilityAgent),
    Tree(TreeHeuristic),
    Flood(FloodHeuristic),
    MonteCarlo(FloodHeuristic),
    Solo(SoloHeuristic),
    Random(RandomAgent),
}

impl Default for Agent {
    fn default() -> Self {
        Self::Mobility(MobilityAgent::default())
    }
}

impl Agent {
    pub async fn step(&self, request: &GameRequest, latency: u64) -> MoveResponse {
        let game = Game::from_request(request);
        let timeout = request.game.timeout.saturating_sub(latency);

        self.step_internal(timeout, &game).await
    }

    pub async fn step_internal(&self, timeout: u64, game: &Game) -> MoveResponse {
        if game.grid.width > MAX_BOARD_SIZE || game.grid.height > MAX_BOARD_SIZE {
            return RandomAgent.step(game).await;
        }

        match self {
            Agent::Mobility(agent) => agent.step(game).await,
            Agent::Tree(agent) => maxn::step(Arc::new(agent.clone()), timeout, game).await,
            Agent::Flood(agent) => maxn::step(Arc::new(agent.clone()), timeout, game).await,
            Agent::MonteCarlo(agent) => mcts::step(Arc::new(agent.clone()), timeout, game).await,
            Agent::Solo(agent) => maxn::step(Arc::new(agent.clone()), timeout, game).await,
            Agent::Random(agent) => agent.step(game).await,
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
