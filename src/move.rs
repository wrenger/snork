use std::path::PathBuf;

use structopt::StructOpt;

mod agents;
use agents::*;
mod env;
mod game;
use game::*;
mod util;

#[derive(structopt::StructOpt)]
enum Opts {
    /// Json data of the request
    Data { data: String },
    /// File containing the data
    File { file: PathBuf },
}

fn main() {
    let request: env::GameRequest = match Opts::from_args() {
        Opts::Data { data } => serde_json::from_str(&data).unwrap(),
        Opts::File { file } => serde_json::from_reader(std::fs::File::open(file).unwrap()).unwrap(),
    };

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

    let agent = request
        .config
        .as_ref()
        .map(|c| c.create_agent(&request))
        .unwrap_or_else(|| Config::default().create_agent(&request));

    let step = agent.lock().unwrap().step(&request, 200);

    println!("Step: {:?}", step);
}
