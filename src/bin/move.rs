use log::info;

use snork::agents::*;
use snork::env::GameRequest;
use snork::floodfill::FloodFill;
use snork::game::*;
use snork::logging;

use clap::Parser;

#[derive(Parser)]
#[clap(version, author, about = "Simulate a move for an agent.")]
struct Opts {
    /// Default configuration.
    #[clap(long, default_value_t)]
    config: Agent,
    /// JSON Game request.
    #[clap(parse(try_from_str = serde_json::from_str))]
    request: GameRequest,
    /// Time in ms that is subtracted from the game timeouts.
    #[clap(long, default_value = "200")]
    latency: usize,
}

#[tokio::main]
async fn main() {
    logging();

    let Opts {
        config,
        request,
        latency,
    } = Opts::parse();

    let game = Game::from_request(&request);
    info!("{config:?}");
    info!("{game:?}");

    let mut flood_fill = FloodFill::new(request.board.width, request.board.height);
    flood_fill.flood_snakes(&game.grid, &game.snakes);
    info!("{flood_fill:?}");

    let step = config.step(&request, latency as _).await;

    info!("Step: {step:?}");
}
