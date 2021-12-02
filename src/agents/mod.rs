use std::str::FromStr;
use std::string::ToString;
use std::sync::Arc;

use tokio::sync::Mutex;

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
pub enum Config {
    Mobility(MobilityConfig),
    Tree(TreeConfig),
    Random,
    Flood(FloodConfig),
}

impl Default for Config {
    fn default() -> Self {
        Config::Tree(TreeConfig::default())
    }
}

impl Config {
    pub fn create_agent(&self, request: &GameRequest) -> Arc<Mutex<Agent>> {
        if request.board.width > 19 || request.board.height > 19 {
            return Arc::new(Mutex::new(Agent::Random(RandomAgent::default())));
        }

        match self {
            Config::Mobility(config) => Arc::new(Mutex::new(Agent::Mobility(MobilityAgent::new(
                request, &config,
            )))),
            Config::Tree(config) => {
                Arc::new(Mutex::new(Agent::Tree(TreeAgent::new(request, &config))))
            }
            Config::Flood(config) => {
                Arc::new(Mutex::new(Agent::Flood(FloodAgent::new(request, &config))))
            }
            _ => Arc::new(Mutex::new(Agent::Random(RandomAgent::default()))),
        }
    }
}

impl FromStr for Config {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ToString for Config {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[derive(Debug)]
pub enum Agent {
    Mobility(MobilityAgent),
    Tree(TreeAgent),
    Random(RandomAgent),
    Flood(FloodAgent),
}

impl Agent {
    pub async fn step<'a>(&mut self, request: &GameRequest, ms: u64) -> MoveResponse {
        match self {
            Agent::Mobility(agent) => agent.step(request, ms).await,
            Agent::Tree(agent) => agent.step(request, ms).await,
            Agent::Random(agent) => agent.step(request, ms).await,
            Agent::Flood(agent) => agent.step(request, ms).await,
        }
    }
}
