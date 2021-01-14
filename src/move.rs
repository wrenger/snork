use std::str::FromStr;

use structopt::StructOpt;

mod agents;
use agents::*;
mod env;
mod game;
use game::*;
mod util;

#[derive(structopt::StructOpt)]
#[structopt(name = "rusty snake move", about = "Simulate a move for an agent.")]
struct Opts {
    /// Default configuration.
    #[structopt(long, default_value)]
    config: Config,
    /// JSON Game request.
    request: env::GameRequest,
}

impl FromStr for env::GameRequest {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

fn main() {
    let Opts { config, request } = Opts::from_args();

    let mut game = Game::new(request.board.width, request.board.height);
    let mut snakes = Vec::with_capacity(4);
    snakes.push(Snake::from(&request.you, 0));
    snakes.extend(
        request
            .board
            .snakes
            .iter()
            .filter(|s| s.id != request.you.id)
            .enumerate()
            .map(|(i, s)| Snake::from(s, i as u8 + 1)),
    );
    game.reset(snakes, &request.board.food);
    println!("{:?}", game.grid);
    let mut flood_fill = FloodFill::new(request.board.width, request.board.height);
    flood_fill.flood_snakes(&game.grid, &game.snakes, 0);
    println!("{:?}", flood_fill);

    let agent = config.create_agent(&request);

    let step = agent.lock().unwrap().step(&request, 200);

    println!("Step: {:?}", step);
}
