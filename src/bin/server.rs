use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use log::{info, warn};
use snork::env::{GameRequest, IndexResponse, API_VERSION};
use snork::{agents::*, logging};

use clap::Parser;
use warp::Filter;

pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runtime server configuration.
struct State {
    latency: u64,
    color: String,
    head: String,
    tail: String,
    author: String,
    config: Agent,
}

/// High performant rust snake.
#[derive(Debug, Parser)]
#[clap(version, author)]
struct Opt {
    /// IP and Port of the webserver.
    ///
    /// **Note**: Use the IP Address of your device if you want to access it from
    /// the outside. (`127.0.0.1` or `localhost` is private to your computer)
    #[clap(long, default_value = "127.0.0.1:5001")]
    host: SocketAddr,
    /// Time in ms that is subtracted from the game timeouts.
    #[clap(long, default_value_t = 100)]
    latency: u64,
    /// Color in hex format.
    #[clap(long, default_value = "#FF7043")]
    color: String,
    /// Head @see https://docs.battlesnake.com/guides/customizations
    #[clap(long, default_value = "sand-worm")]
    head: String,
    /// Tail @see https://docs.battlesnake.com/guides/customizations
    #[clap(long, default_value = "pixel")]
    tail: String,
    /// Profile name of the battlesnake account
    #[clap(long, default_value = "wrenger")]
    author: String,
    /// Default configuration.
    #[clap(long, default_value_t)]
    config: Agent,
}

#[tokio::main]
async fn main() {
    logging();

    let Opt {
        host,
        latency,
        color,
        head,
        tail,
        author,
        config,
    } = Opt::parse();

    let state = Arc::new(State {
        latency,
        color,
        head,
        tail,
        author,
        config,
    });

    let index = warp::get()
        .and(warp::path::end())
        .and(with_state(state.clone()))
        .map(|state: Arc<State>| {
            warn!("index");
            warp::reply::json(&IndexResponse::new(
                API_VERSION,
                &state.author,
                &state.color,
                &state.head,
                &state.tail,
                PACKAGE_VERSION,
            ))
        });

    let start = warp::path("start")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request: GameRequest| {
            warn!("start {request}");
            warp::reply()
        });

    let r#move = warp::path("move")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(step);

    let end = warp::path("end")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request: GameRequest| {
            warn!("end {request}");
            warp::reply()
        });

    warp::serve(index.or(start).or(r#move).or(end))
        .run(host)
        .await;
}

fn with_state(
    config: Arc<State>,
) -> impl Filter<Extract = (Arc<State>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

async fn step(request: GameRequest, state: Arc<State>) -> Result<impl warp::Reply, Infallible> {
    warn!("move {request}");

    let timer = Instant::now();
    let next_move = state.config.step(&request, state.latency).await;
    info!("{next_move:?}, {:?}ms", timer.elapsed().as_millis());

    Ok(warp::reply::json(&next_move))
}
