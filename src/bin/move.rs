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
    #[clap(long, default_value_t, value_parser)]
    config: Agent,
    /// JSON Game request.
    #[clap(value_parser = parse_request)]
    request: GameRequest,
    /// Time in ms that is subtracted from the game timeouts.
    #[clap(long, default_value_t = 200, value_parser)]
    latency: usize,
}

fn parse_request(s: &str) -> Result<GameRequest, serde_json::Error> {
    serde_json::from_str(s)
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
