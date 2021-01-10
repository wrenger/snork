use std::path::PathBuf;

use structopt::StructOpt;

mod agents;
use agents::*;
mod env;
use env::*;
mod game;
mod util;

#[derive(structopt::StructOpt)]
enum Opts {
    /// Json data of the request
    Data { data: String },
    /// File containing the data
    File { file: PathBuf },
}

fn main() {
    let request: GameRequest = match Opts::from_args() {
        Opts::Data { data } => serde_json::from_str(&data).unwrap(),
        Opts::File { file } => serde_json::from_reader(std::fs::File::open(file).unwrap()).unwrap(),
    };

    let agent = request
        .config
        .as_ref()
        .map(|c| c.create_agent(&request))
        .unwrap_or_else(|| Config::default().create_agent(&request));

    let step = agent.lock().unwrap().step(&request, 200);

    println!("Step: {:?}", step);
}
