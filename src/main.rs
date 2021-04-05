use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chashmap::CHashMap;

use snork_core::agents::*;
use snork_core::env::{GameRequest, IndexResponse, MoveResponse};

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
    agent: Arc<Mutex<dyn Agent + Send + 'static>>,
    start_time: Instant,
}

impl RunningInstance {
    fn new(agent: Arc<Mutex<dyn Agent + Send + 'static>>) -> RunningInstance {
        RunningInstance {
            agent,
            start_time: Instant::now(),
        }
    }
}

/// Runtime server configuration.
struct ServerConfig {
    runtime: u64,
    color: String,
    head: String,
    tail: String,
    config: Config,
    running_agents: Arc<CHashMap<(String, String), RunningInstance>>,
    save_queue: Option<Sender<Option<GameRequest>>>,
}

impl ServerConfig {
    fn new(
        runtime: u64,
        color: String,
        head: String,
        tail: String,
        config: Config,
        running_agents: Arc<CHashMap<(String, String), RunningInstance>>,
        save_queue: Option<Sender<Option<GameRequest>>>,
    ) -> ServerConfig {
        ServerConfig {
            runtime,
            color,
            head,
            tail,
            config,
            running_agents,
            save_queue,
        }
    }

    fn clear_agents(&self) {
        if !self.running_agents.is_empty() {
            let now = Instant::now();
            self.running_agents
                .retain(|_, v| (now - v.start_time) < MAX_RUNTIME);
        }
        println!("{} instances running", self.running_agents.len());
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
async fn start(data: web::Data<ServerConfig>, request: web::Json<GameRequest>) -> HttpResponse {
    println!(
        "start {} game {},{}",
        request.game.ruleset.name, request.game.id, request.you.id
    );
    data.clear_agents();

    if data.running_agents.len() < MAX_AGENT_COUNT {
        data.running_agents.insert(
            (request.game.id.clone(), request.you.id.clone()),
            RunningInstance::new(data.config.create_agent(&request)),
        );
    }
    HttpResponse::Ok().body("")
}

#[post("/move")]
async fn game_move(data: web::Data<ServerConfig>, request: web::Json<GameRequest>) -> HttpResponse {
    println!(
        "move {} game {},{}",
        request.game.ruleset.name, request.game.id, request.you.id
    );

    if let Some(instance) = data
        .running_agents
        .get(&(request.game.id.clone(), request.you.id.clone()))
    {
        let timer = Instant::now();
        let next_move = instance.agent.lock().unwrap().step(&request, data.runtime);
        if let Some(save_queue) = &data.save_queue {
            save_queue.send(Some(request.into_inner())).unwrap();
        }
        println!("response time {:?}ms", (Instant::now() - timer).as_millis());
        HttpResponse::Ok().json(next_move)
    } else {
        HttpResponse::Ok().json(MoveResponse::default())
    }
}

#[post("/end")]
async fn end(data: web::Data<ServerConfig>, request: web::Json<GameRequest>) -> HttpResponse {
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

    data.clear_agents();
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

    let save_queue = log_dir.map(snork_core::savegame::worker);
    let running_agents = Arc::new(CHashMap::new());

    let save_queue_copy = save_queue.clone();
    let result = HttpServer::new(move || {
        App::new()
            .data(ServerConfig::new(
                runtime,
                color.clone(),
                head.clone(),
                tail.clone(),
                config.clone(),
                running_agents.clone(),
                save_queue_copy.clone(),
            ))
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
