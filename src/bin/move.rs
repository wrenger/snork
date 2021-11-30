use structopt::StructOpt;

use snork::env::GameRequest;
use snork::agents::*;
use snork::game::*;

#[derive(structopt::StructOpt)]
#[structopt(name = "rusty snake move", about = "Simulate a move for an agent.")]
struct Opts {
    /// Default configuration.
    #[structopt(long, default_value)]
    config: Config,
    /// JSON Game request.
    #[structopt(parse(try_from_str = serde_json::from_str))]
    request: GameRequest,
    #[structopt(long, default_value = "200")]
    runtime: usize,
}

fn main() {
    let Opts {
        config,
        request,
        runtime,
    } = Opts::from_args();

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
    game.reset(snakes, &request.board.food, &request.board.hazards);
    println!("{:?}", game);
    let mut flood_fill = FloodFill::new(request.board.width, request.board.height);
    flood_fill.flood_snakes(&game.grid, &game.snakes);
    println!("{:?}", flood_fill);

    let agent = config.create_agent(&request);

    let step = agent.lock().unwrap().step(&request, runtime as _);

    println!("Step: {:?}", step);
}
