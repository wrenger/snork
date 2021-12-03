use std::str::FromStr;
use std::string::ToString;

mod mobility;
pub use mobility::*;
mod flood;
pub use flood::*;
mod random;
pub use random::*;
mod tree;
pub use tree::*;

use super::env::{GameRequest, MoveResponse};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Agent {
    Mobility(MobilityAgent),
    Tree(TreeAgent),
    Flood(FloodAgent),
    Random(RandomAgent),
}

impl Default for Agent {
    fn default() -> Self {
        Self::Mobility(MobilityAgent::default())
    }
}

impl Agent {
    pub async fn step<'a>(&self, request: &GameRequest, ms: u64) -> MoveResponse {
        if request.board.width > 19 || request.board.height > 19 {
            return RandomAgent.step(request, ms).await;
        }

        match self {
            Agent::Mobility(agent) => agent.step(request, ms).await,
            Agent::Tree(agent) => agent.step(request, ms).await,
            Agent::Flood(agent) => agent.step(request, ms).await,
            Agent::Random(agent) => agent.step(request, ms).await,
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
