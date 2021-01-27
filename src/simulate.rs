use structopt::StructOpt;

mod agents;
use agents::*;
mod env;
use env::*;
mod game;
use game::*;
mod util;

use rand::prelude::*;
use rand::seq::IteratorRandom;
use std::time::Instant;

use std::sync::mpsc;
use threadpool::ThreadPool;

#[derive(structopt::StructOpt)]
#[structopt(
    name = "rusty snake simulator",
    about = "Simulate a game with different agents."
)]
struct Opts {
    #[structopt(long, default_value = "200")]
    runtime: usize,
    #[structopt(long, default_value = "11")]
    width: usize,
    #[structopt(long, default_value = "11")]
    height: usize,
    #[structopt(long, default_value = "0.15")]
    food_rate: f64,
    #[structopt(short, long, default_value = "1")]
    game_count: usize,
    #[structopt(short, long, default_value = "4")]
    jobs: usize,
    #[structopt(short, long)]
    verbose: bool,

    agents: Vec<Config>,
}

fn init_game(width: usize, height: usize, num_agents: usize) -> Game {
    let mut rng = rand::thread_rng();
    let start_positions = (0..width * height)
        .filter(|i| i % 2 == 0)
        .map(|i| Vec2D::new((i % width) as i16, (i / width) as i16))
        .choose_multiple(&mut rng, num_agents);

    let snakes = start_positions
        .iter()
        .enumerate()
        .map(|(i, p)| Snake::new(i as _, vec![*p; 3].into(), 100))
        .collect();

    let mut game = Game::new(width, height);
    game.reset(snakes, &[]);

    // Spawn 1 food 2 steps away from each snake
    for snake in game.snakes.clone() {
        let p = [
            Vec2D::new(-1, -1),
            Vec2D::new(-2, 0),
            Vec2D::new(-1, 1),
            Vec2D::new(0, 2),
            Vec2D::new(1, 1),
            Vec2D::new(2, 0),
            Vec2D::new(1, -1),
            Vec2D::new(0, -2),
        ]
        .iter()
        .map(|&p| snake.head() + p)
        .filter(|&p| game.grid.has(p) && game.grid[p] == Cell::Free)
        .choose(&mut rng);
        if let Some(p) = p {
            game.grid[p] = Cell::Food;
        }
    }

    game
}

fn snake_data(s: &Snake) -> SnakeData {
    SnakeData {
        id: format!("{}", s.id),
        name: String::new(),
        health: s.health,
        body: s.body.iter().cloned().rev().collect(),
        shout: String::new(),
    }
}

fn game_to_request(game: &Game, turn: usize) -> GameRequest {
    let snakes = game.snakes.iter().map(snake_data).collect::<Vec<_>>();

    let food = game
        .grid
        .cells
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if *c == Cell::Food {
                Some(Vec2D::new(
                    (i % game.grid.width) as i16,
                    (i / game.grid.width) as i16,
                ))
            } else {
                None
            }
        })
        .collect();

    GameRequest {
        game: GameData::default(),
        turn: turn as _,
        you: snakes[0].clone(),
        board: Board {
            height: game.grid.height,
            width: game.grid.width,
            food,
            hazards: Vec::new(),
            snakes,
        },
    }
}

fn play_game(
    agents: &[Config],
    width: usize,
    height: usize,
    runtime: usize,
    food_rate: f64,
    verbose: bool,
) -> bool {
    let mut rng = rand::thread_rng();
    let mut game = init_game(width, height, agents.len());
    let mut request = game_to_request(&game, 0);
    let agents = agents
        .iter()
        .enumerate()
        .map(|(i, c)| {
            request.you = request.board.snakes[i].clone();
            c.create_agent(&request)
        })
        .collect::<Vec<_>>();

    for turn in 0.. {
        if verbose {
            println!("{} {:?}", turn, game.grid);
        }

        let mut request = game_to_request(&game, turn);
        let mut moves = [Direction::Up; 4];
        for snake in &game.snakes {
            if snake.alive() {
                request.you = snake_data(snake);
                moves[snake.id as usize] = agents[snake.id as usize]
                    .lock()
                    .unwrap()
                    .step(&request, runtime as _)
                    .r#move;
            }
        }
        if verbose {
            println!("Moves: {:?}", moves);
        }
        game.step(&moves);

        if game.outcome() == Outcome::Winner(0) {
            println!("game: win after {} turns", turn);
            return true;
        }
        if !game.snake_is_alive(0) {
            println!("game: loss after {} turns", turn);
            return false;
        }

        // Spawn food
        if request.board.food.is_empty() || rng.gen::<f64>() < food_rate {
            if let Some(cell) = game
                .grid
                .cells
                .iter_mut()
                .filter(|c| **c == Cell::Free)
                .choose(&mut rng)
            {
                *cell = Cell::Food;
            }
        }
    }
    false
}

fn main() {
    let Opts {
        runtime,
        width,
        height,
        food_rate,
        game_count,
        jobs,
        verbose,
        agents,
    } = Opts::from_args();

    assert!(agents.len() <= 4, "Only up to 4 snakes are supported");

    let start = Instant::now();

    let pool = ThreadPool::new(jobs);
    let (tx, rx) = mpsc::channel();
    for _ in 0..game_count {
        let tx = tx.clone();
        let agents = agents.clone();
        pool.execute(move || {
            tx.send(play_game(
                &agents, width, height, runtime, food_rate, verbose,
            ))
            .unwrap();
        })
    }
    drop(tx);

    let wins = rx.iter().filter(|x| *x).count();

    println!(
        "Simulation time: {}ms",
        (Instant::now() - start).as_millis()
    );
    println!("Result: {}/{}", wins, game_count);
}
