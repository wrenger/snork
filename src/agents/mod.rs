use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    fn step(&mut self, request: &GameRequest, ms: u64) -> MoveResponse;
    fn end(&mut self, request: &GameRequest);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Config {
    Mobility(MobilityConfig),
    Tree(TreeConfig),
    Random,
}

impl Default for Config {
    fn default() -> Self {
        Config::Tree(TreeConfig::default())
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

impl FromStr for Config {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ToString for Config {
    fn to_string(&self) -> std::string::String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[ignore]
    fn test_move() {
        use super::*;
        use std::path::PathBuf;
        use structopt::StructOpt;

        #[derive(structopt::StructOpt)]
        enum Opts {
            /// Json data of the request
            Data { data: String },
            /// File containing the data
            File { file: PathBuf },
        }

        let request: GameRequest = match Opts::from_args() {
            Opts::Data { data } => serde_json::from_str(&data).unwrap(),
            Opts::File { file } => {
                serde_json::from_reader(std::fs::File::open(file).unwrap()).unwrap()
            }
        };

        let agent = request
            .config
            .as_ref()
            .map(|c| c.create_agent(&request))
            .unwrap_or_else(|| Config::default().create_agent(&request));

        let step = agent.lock().unwrap().step(&request, 200);

        println!("Step: {:?}", step);
    }
}
