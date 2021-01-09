use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chashmap::CHashMap;

mod env;
use env::{GameRequest, IndexResponse, MoveResponse};

mod util;

mod agents;
use agents::*;

mod game;
mod savegame;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use structopt::StructOpt;

pub const API_VERSION: &str = "1";
pub const AUTHOR: &str = "l4r0x";

/// Max number of parallel agent instances
pub const MAX_AGENT_COUNT: usize = 10;
/// Max runtime an agent has before it is forcefully terminated
pub const MAX_RUNTIME: Duration = Duration::from_secs(60 * 10);

/// Running instance of an agent
#[derive(Debug)]
struct RunningInstance {
    agent: Arc<Mutex<dyn Agent + Send>>,
    start_time: Instant,
}

impl RunningInstance {
    fn new(agent: Arc<Mutex<dyn Agent + Send>>) -> RunningInstance {
        RunningInstance {
            agent,
            start_time: Instant::now(),
        }
    }
}

struct ServerConfig {
    runtime: u64,
    save_queue: Option<Sender<Option<GameRequest>>>,
    color: String,
    head: String,
    tail: String,
    config: Config,
}

impl ServerConfig {
    fn new(
        runtime: u64,
        save_queue: Option<Sender<Option<GameRequest>>>,
        color: String,
        head: String,
        tail: String,
        config: Config,
    ) -> ServerConfig {
        ServerConfig {
            runtime,
            save_queue,
            color,
            head,
            tail,
            config,
        }
    }
}

/// Container holding the server state and running agents
#[derive(Debug)]
struct ServerData {
    running_agents: CHashMap<(String, String), RunningInstance>,
}

impl ServerData {
    fn new() -> ServerData {
        ServerData {
            running_agents: CHashMap::new(),
        }
    }
}

#[get("/")]
async fn index(config: web::Data<ServerConfig>) -> HttpResponse {
    println!("index");
    HttpResponse::Ok().json(IndexResponse::new(
        API_VERSION,
        AUTHOR,
        config.color.clone(),
        config.head.clone(),
        config.tail.clone(),
    ))
}

#[post("/start")]
async fn start(
    config: web::Data<ServerConfig>,
    data: web::Data<ServerData>,
    request: web::Json<GameRequest>,
) -> HttpResponse {
    println!(
        "start {} game {},{}",
        request.game.ruleset.name, request.game.id, request.you.id
    );
    if !data.running_agents.is_empty() {
        let now = Instant::now();
        data.running_agents
            .retain(|_, v| (now - v.start_time) < MAX_RUNTIME);
    }

    if data.running_agents.len() < MAX_AGENT_COUNT {
        data.running_agents.insert(
            (request.game.id.clone(), request.you.id.clone()),
            RunningInstance::new(if let Some(config) = &request.config {
                config.create_agent(&request)
            } else {
                config.config.create_agent(&request)
            }),
        );
    }
    println!("{} instances running", data.running_agents.len());
    HttpResponse::Ok().body("")
}

#[post("/move")]
async fn game_move(
    config: web::Data<ServerConfig>,
    data: web::Data<ServerData>,
    request: web::Json<GameRequest>,
) -> HttpResponse {
    println!(
        "move {} game {},{}",
        request.game.ruleset.name, request.game.id, request.you.id
    );

    if let Some(instance) = data
        .running_agents
        .get(&(request.game.id.clone(), request.you.id.clone()))
    {
        let timer = Instant::now();
        let next_move = instance
            .agent
            .lock()
            .unwrap()
            .step(&request, config.runtime);
        if let Some(save_queue) = &config.save_queue {
            save_queue.send(Some(request.into_inner())).unwrap();
        }
        println!("response time {:?}ms", (Instant::now() - timer).as_millis());
        HttpResponse::Ok().json(next_move)
    } else {
        HttpResponse::Ok().json(MoveResponse::default())
    }
}

#[post("/end")]
async fn end(data: web::Data<ServerData>, request: web::Json<GameRequest>) -> HttpResponse {
    println!(
        "end {} game {},{}",
        request.game.ruleset.name, request.game.id, request.you.id
    );

    if let Some(instance) = data
        .running_agents
        .get(&(request.game.id.clone(), request.you.id.clone()))
    {
        instance.agent.lock().unwrap().end(&request);
    }
    data.running_agents
        .remove(&(request.game.id.clone(), request.you.id.clone()));

    if !data.running_agents.is_empty() {
        let now = Instant::now();
        data.running_agents
            .retain(|_, v| (now - v.start_time) < MAX_RUNTIME);
    }
    println!("{} instances running", data.running_agents.len());
    HttpResponse::Ok().body("")
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rusty snake", about = "High performant rust snake.")]
struct Opt {
    /// Port of the webserver.
    #[structopt(short, long, default_value = "5001")]
    port: u16,
    /// Time per step in ms.
    #[structopt(long, default_value = "200")]
    runtime: u64,
    /// Directory where games are logged.
    #[structopt(short, long)]
    log_dir: Option<PathBuf>,

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let Opt {
        port,
        runtime,
        log_dir,
        color,
        head,
        tail,
        config,
    } = Opt::from_args();
    let save_queue = log_dir.map(savegame::worker);
    let server_data = web::Data::new(ServerData::new());

    let save_queue_copy = save_queue.clone();
    let result = HttpServer::new(move || {
        App::new()
            .data(ServerConfig::new(
                runtime,
                save_queue_copy.clone(),
                color.clone(),
                head.clone(),
                tail.clone(),
                config.clone(),
            ))
            .app_data(server_data.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                println!("ERROR: {}", err);
                actix_web::error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(r#"{{"error":"{}"}}"#, err)),
                )
                .into()
            }))
            .service(index)
            .service(start)
            .service(game_move)
            .service(end)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await;

    if let Some(save_queue) = save_queue {
        save_queue.send(None).unwrap();
    }

    result
}
