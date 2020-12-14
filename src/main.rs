use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chashmap::CHashMap;

mod env;
use env::{GameRequest, IndexResponse, MoveResponse};

mod agents;
use agents::{Agent, EatAllAgent, Random};

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use structopt::StructOpt;

pub const API_VERSION: &str = "1";
pub const AUTHOR: &str = "l4r0x";
pub const COLOR: &str = "#FF7043";
pub const HEAD: &str = "sand-worm";
pub const TAIL: &str = "pixel";

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

/// Container holding the server state and running agents
#[derive(Debug, Default)]
struct ServerData {
    running_agents: CHashMap<(String, String), RunningInstance>,
}

#[get("/")]
async fn index() -> HttpResponse {
    println!("index");
    HttpResponse::Ok().json(IndexResponse::new(API_VERSION, AUTHOR, COLOR, HEAD, TAIL))
}

#[post("/start")]
async fn start(data: web::Data<ServerData>, reqest: web::Json<GameRequest>) -> HttpResponse {
    println!(
        "start {} game {},{}",
        reqest.game.ruleset.name, reqest.game.id, reqest.you.id
    );

    if data.running_agents.len() < MAX_AGENT_COUNT {
        let agent: Arc<Mutex<dyn Agent + Send>> = if reqest.game.ruleset.name == "standard" {
            let mut agent = EatAllAgent::default();
            agent.start(&reqest);
            Arc::new(Mutex::new(agent))
        } else {
            let mut agent = Random::default();
            agent.start(&reqest);
            Arc::new(Mutex::new(agent))
        };
        data.running_agents.insert(
            (reqest.game.id.clone(), reqest.you.id.clone()),
            RunningInstance::new(agent),
        );
    }

    HttpResponse::Ok().body("")
}

#[post("/move")]
async fn game_move(data: web::Data<ServerData>, reqest: web::Json<GameRequest>) -> HttpResponse {
    println!(
        "move {} game {},{}",
        reqest.game.ruleset.name, reqest.game.id, reqest.you.id
    );

    if let Some(instance) = data
        .running_agents
        .get(&(reqest.game.id.clone(), reqest.you.id.clone()))
    {
        return HttpResponse::Ok().json(instance.agent.lock().unwrap().step(&reqest));
    }
    HttpResponse::Ok().json(MoveResponse::default())
}

#[post("/end")]
async fn end(data: web::Data<ServerData>, reqest: web::Json<GameRequest>) -> HttpResponse {
    println!(
        "end {} game {},{}",
        reqest.game.ruleset.name, reqest.game.id, reqest.you.id
    );

    if let Some(instance) = data
        .running_agents
        .get(&(reqest.game.id.clone(), reqest.you.id.clone()))
    {
        instance.agent.lock().unwrap().end(&reqest);
    }
    data.running_agents
        .remove(&(reqest.game.id.clone(), reqest.you.id.clone()));

    let now = Instant::now();
    data.running_agents
        .retain(|_, v| (now - v.start_time) < MAX_RUNTIME);
    HttpResponse::Ok().body("")
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rusty snake", about = "High performant rust snake.")]
struct Opt {
    /// Port of the webserver.
    #[structopt(short, long, default_value = "5001")]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let data = web::Data::new(ServerData::default());

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(index)
            .service(start)
            .service(game_move)
            .service(end)
    })
    .bind(("0.0.0.0", opt.port))?
    .run()
    .await
}
