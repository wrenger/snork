use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chashmap::CHashMap;

use snork::agents::*;
use snork::env::{GameRequest, IndexResponse, MoveResponse, API_VERSION};

use structopt::StructOpt;
use tokio::sync::Mutex;
use warp::Filter;

pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "l4r0x";

/// Max number of parallel agent instances
pub const MAX_AGENT_COUNT: usize = 10;
/// Max runtime an agent has before it is forcefully terminated
pub const MAX_RUNTIME: Duration = Duration::from_secs(60 * 10);

/// Running instance of an agent
#[derive(Debug)]
struct RunningInstance {
    agent: Arc<tokio::sync::Mutex<Agent>>,
    start_time: Instant,
}

impl RunningInstance {
    fn new(agent: Arc<Mutex<Agent>>) -> RunningInstance {
        RunningInstance {
            agent,
            start_time: Instant::now(),
        }
    }
}

/// Runtime server configuration.
struct State {
    runtime: u64,
    color: String,
    head: String,
    tail: String,
    config: Config,
    agents: CHashMap<(String, String), RunningInstance>,
}

impl State {
    fn new(runtime: u64, color: String, head: String, tail: String, config: Config) -> State {
        State {
            runtime,
            color,
            head,
            tail,
            config,
            agents: CHashMap::new(),
        }
    }

    fn clear_agents(&self) {
        if !self.agents.is_empty() {
            let now = Instant::now();
            self.agents
                .retain(|_, v| (now - v.start_time) < MAX_RUNTIME);
        }
        println!("{} instances running", self.agents.len());
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rusty snake", about = "High performant rust snake.")]
struct Opt {
    /// Port of the webserver.
    #[structopt(long, default_value = "127.0.0.1:5001")]
    host: SocketAddr,
    /// Time per step in ms.
    #[structopt(long, default_value = "200")]
    runtime: u64,
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
    config: Config,
}

#[tokio::main]
async fn main() {
    let Opt {
        host,
        runtime,
        color,
        head,
        tail,
        config,
    } = Opt::from_args();

    let state = Arc::new(State::new(runtime, color, head, tail, config));

    let index = warp::get()
        .and(warp::path::end())
        .and(with_state(state.clone()))
        .map(|state: Arc<State>| {
            println!("index");
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
        .and(with_state(state.clone()))
        .and(warp::post())
        .and(warp::body::json::<GameRequest>())
        .map(|state: Arc<State>, request: GameRequest| {
            println!(
                "start {} game {},{}",
                request.game.ruleset.name, request.game.id, request.you.id
            );
            state.clear_agents();

            if state.agents.len() < MAX_AGENT_COUNT {
                state.agents.insert(
                    (request.game.id.clone(), request.you.id.clone()),
                    RunningInstance::new(state.config.create_agent(&request)),
                );
            }
            warp::reply()
        });

    let r#move = warp::path("move")
        .and(with_state(state.clone()))
        .and(warp::post())
        .and(warp::body::json::<GameRequest>())
        .and_then(step);

    let end = warp::path("end")
        .and(with_state(state.clone()))
        .and(warp::post())
        .and(warp::body::json::<GameRequest>())
        .map(|state: Arc<State>, request: GameRequest| {
            println!(
                "end {} game {},{}",
                request.game.ruleset.name, request.game.id, request.you.id
            );

            state
                .agents
                .remove(&(request.game.id.clone(), request.you.id.clone()));

            state.clear_agents();
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
    println!(
        "move {} game {},{}",
        request.game.ruleset.name, request.game.id, request.you.id
    );

    if let Some(instance) = state
        .agents
        .get(&(request.game.id.clone(), request.you.id.clone()))
    {
        let timer = Instant::now();

        let mut agent = instance.agent.lock().await;
        let next_move = agent.step(&request, state.runtime).await;

        println!("response time {:?}ms", (Instant::now() - timer).as_millis());
        Ok(warp::reply::json(&next_move))
    } else {
        Ok(warp::reply::json(&MoveResponse::default()))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn chashmap() {
        use chashmap::CHashMap;
        use std::sync::Arc;

        let map = Arc::new(CHashMap::new());

        let clone = map.clone();
        clone.insert(String::from("hello"), 3);

        assert!(map.get("hello").map(|v| *v == 3).unwrap_or_default());
    }
}
