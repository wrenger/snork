use log::info;
use structopt::StructOpt;

use snork::agents::*;
use snork::env::GameRequest;
use snork::game::*;
use snork::logging;

#[derive(structopt::StructOpt)]
#[structopt(name = "rusty snake move", about = "Simulate a move for an agent.")]
struct Opts {
    /// Default configuration.
    #[structopt(long, default_value)]
    config: Agent,
    /// JSON Game request.
    #[structopt(parse(try_from_str = serde_json::from_str))]
    request: GameRequest,
    #[structopt(long, default_value = "200")]
    runtime: usize,
}

#[tokio::main]
async fn main() {
    logging();

    let Opts {
        config,
        request,
        runtime,
    } = Opts::from_args();

    let game = Game::from_request(&request);
    info!("{:?}", game);

    let mut flood_fill = FloodFill::new(request.board.width, request.board.height);
    flood_fill.flood_snakes(&game.grid, &game.snakes);
    info!("{:?}", flood_fill);

    let step = config.step(&request, runtime as _).await;

    info!("Step: {:?}", step);
}
