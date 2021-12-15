use log::info;
use structopt::StructOpt;

use snork::agents::*;
use snork::env::GameRequest;
use snork::game::*;
use snork::logging;

#[derive(structopt::StructOpt)]
#[structopt(name = "snork move", about = "Simulate a move for an agent.")]
struct Opts {
    /// Default configuration.
    #[structopt(long, default_value)]
    config: Agent,
    /// JSON Game request.
    #[structopt(parse(try_from_str = serde_json::from_str))]
    request: GameRequest,
    /// Time in ms that is subtracted from the game timeouts.
    #[structopt(long, default_value = "200")]
    latency: usize,
}

#[tokio::main]
async fn main() {
    logging();

    let Opts {
        config,
        request,
        latency,
    } = Opts::from_args();

    let game = Game::from_request(&request);
    info!("{:?}", game);

    let mut flood_fill = FloodFill::new(request.board.width, request.board.height);
    flood_fill.flood_snakes(&game.grid, &game.snakes);
    info!("{:?}", flood_fill);

    let step = config.step(&request, latency as _).await;

    info!("Step: {:?}", step);
}
