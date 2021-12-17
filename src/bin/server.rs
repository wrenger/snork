use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use log::{info, warn};
use snork::agents::*;
use snork::env::{GameRequest, IndexResponse, API_VERSION};

use structopt::StructOpt;
use warp::Filter;

pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "l4r0x";

/// Runtime server configuration.
struct State {
    latency: u64,
    color: String,
    head: String,
    tail: String,
    config: Agent,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "snork server", about = "High performant rust snake.")]
struct Opt {
    /// IP and Port of the webserver.
    /// **Note**: Use the IP Address of your device if you want to access it from another device. (`127.0.0.1` or `localhost` is private to your computer)
    #[structopt(long, default_value = "127.0.0.1:5001")]
    host: SocketAddr,
    /// Time in ms that is subtracted from the game timeouts.
    #[structopt(long, default_value = "100")]
    latency: u64,
    /// Color in hex format.
    #[structopt(long, default_value = "#FF7043")]
    color: String,
    /// Head @see https://docs.battlesnake.com/references/personalization
    #[structopt(long, default_value = "sand-worm")]
    head: String,
    /// Tail @see https://docs.battlesnake.com/references/personalization
    #[structopt(long, default_value = "pixel")]
    tail: String,
    /// Default configuration.
    #[structopt(long, default_value)]
    config: Agent,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let Opt {
        host,
        latency,
        color,
        head,
        tail,
        config,
    } = Opt::from_args();

    let state = Arc::new(State {
        latency,
        color,
        head,
        tail,
        config,
    });

    let index = warp::get()
        .and(warp::path::end())
        .and(with_state(state.clone()))
        .map(|state: Arc<State>| {
            warn!("index");
            warp::reply::json(&IndexResponse::new(
                API_VERSION.into(),
                AUTHOR.into(),
                state.color.clone().into(),
                state.head.clone().into(),
                state.tail.clone().into(),
                PACKAGE_VERSION.into(),
            ))
        });

    let start = warp::path("start")
        .and(warp::post())
        .and(warp::body::json::<GameRequest>())
        .map(|request: GameRequest| {
            warn!(
                "start {} game {},{}",
                request.game.ruleset.name, request.game.id, request.you.id
            );
            warp::reply()
        });

    let r#move = warp::path("move")
        .and(with_state(state.clone()))
        .and(warp::post())
        .and(warp::body::json::<GameRequest>())
        .and_then(step);

    let end = warp::path("end")
        .and(warp::post())
        .and(warp::body::json::<GameRequest>())
        .map(|request: GameRequest| {
            warn!(
                "end {} game {},{} win={}",
                request.game.ruleset.name,
                request.game.id,
                request.you.id,
                request.you.health != 0
            );
            warp::reply()
        });

    warp::serve(index.or(start).or(r#move).or(end))
        .run(host)
        .await
}

fn with_state(
    config: Arc<State>,
) -> impl Filter<Extract = (Arc<State>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

async fn step(state: Arc<State>, request: GameRequest) -> Result<impl warp::Reply, Infallible> {
    warn!(
        "move {} game {},{}",
        request.game.ruleset.name, request.game.id, request.you.id
    );

    let timer = Instant::now();
    let next_move = state.config.step(&request, state.latency).await;
    info!("response time {:?}ms", timer.elapsed().as_millis());

    Ok(warp::reply::json(&next_move))
}
