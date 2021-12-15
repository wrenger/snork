use log::{debug, warn};
use owo_colors::OwoColorize;
use snork::logging;
use structopt::StructOpt;

use snork::agents::*;
use snork::env::*;
use snork::game::*;

use rand::prelude::*;
use rand::seq::IteratorRandom;
use std::time::Instant;

#[derive(structopt::StructOpt)]
#[structopt(
    name = "rusty snake simulator",
    about = "Simulate a game with different agents."
)]
struct Opts {
    #[structopt(long, default_value = "200")]
    timeout: u64,
    #[structopt(long, default_value = "11")]
    width: usize,
    #[structopt(long, default_value = "11")]
    height: usize,
    #[structopt(long, default_value = "0.15")]
    food_rate: f64,
    #[structopt(short, long, default_value = "25")]
    shrink_turns: usize,
    #[structopt(short, long, default_value = "1")]
    game_count: usize,

    agents: Vec<Agent>,
}

#[tokio::main]
async fn main() {
    logging();

    let Opts {
        timeout,
        width,
        height,
        food_rate,
        shrink_turns,
        game_count,
        agents,
    } = Opts::from_args();

    assert!(agents.len() <= 4, "Only up to 4 snakes are supported");

    let start = Instant::now();

    let mut wins = 0;

    for i in 0..game_count {
        let win = play_game(&agents, width, height, timeout, food_rate, shrink_turns).await;
        wins += win as usize;
        warn!(
            "{}: {} {}ms",
            "Finish Game".bright_green(),
            i,
            start.elapsed().as_millis()
        );
    }

    println!("Result: {}/{}", wins, game_count);
}

async fn play_game(
    agents: &[Agent],
    width: usize,
    height: usize,
    timeout: u64,
    food_rate: f64,
    shrink_turns: usize,
) -> bool {
    let mut game = init_game(width, height, agents.len());
    let mut food_count = 4;
    let mut rng = rand::thread_rng();

    debug!("init: {:?}", game);

    let mut hazard_insets = [0; 4];

    for turn in 0.. {
        let mut moves = [Direction::Up; 4];
        for i in 0..game.snakes.len() {
            if game.snakes[i].alive() {
                // Agents assume player 0 is you.
                game.snakes.swap(0, i);

                let response = agents[i].step_internal(timeout, &game).await;
                moves[i] = response.r#move;

                game.snakes.swap(0, i);
            }
        }
        debug!("Moves: {:?}", moves);

        game.step(&moves);

        debug!("{}: {:?}", turn, game);

        if !game.snake_is_alive(0) {
            warn!("game: loss after {} turns", turn);
            return false;
        }
        if game.outcome() == Outcome::Winner(0) {
            warn!("game: win after {} turns", turn);
            return true;
        }

        // Check if snakes have consumed food
        for snake in &game.snakes {
            if snake.alive() && snake.health == 100 {
                food_count -= 1;
            }
        }

        // Spawn food
        if food_count == 0 || rng.gen::<f64>() < food_rate {
            if let Some(cell) = game
                .grid
                .cells
                .iter_mut()
                .filter(|c| !c.food() && !c.owned())
                .choose(&mut rng)
            {
                cell.set_food(true);
                food_count += 1;
            }
        }

        // Hazards
        if turn > 0
            && turn % shrink_turns == 0
            && hazard_insets[0] + hazard_insets[2] < game.grid.height
            && hazard_insets[1] + hazard_insets[3] < game.grid.width
        {
            let dir = rng.gen_range(0..4);
            hazard_insets[dir] += 1;
            if dir % 2 == 0 {
                let y = if dir == 0 {
                    hazard_insets[dir] - 1
                } else {
                    game.grid.height - hazard_insets[dir]
                };
                for x in 0..game.grid.width {
                    game.grid[v2(x as _, y as _)].set_hazard(true);
                }
            } else {
                let x = if dir == 1 {
                    hazard_insets[dir] - 1
                } else {
                    game.grid.width - hazard_insets[dir]
                };
                for y in 0..game.grid.height {
                    game.grid[v2(x as _, y as _)].set_hazard(true);
                }
            }
        }
    }
    false
}

fn init_game(width: usize, height: usize, num_agents: usize) -> Game {
    let mut rng = rand::thread_rng();
    let start_positions = (0..width * height)
        .filter(|i| i % 2 == 0)
        .map(|i| v2((i % width) as i16, (i / width) as i16))
        .choose_multiple(&mut rng, num_agents);

    let snakes = start_positions
        .into_iter()
        .map(|p| Snake::new(vec![p; 3].into(), 100))
        .collect();

    let mut game = Game::new(0, width, height, snakes, &[], &[]);

    // Spawn 1 food 2 steps away from each snake
    for snake in game.snakes.clone() {
        let p = [
            v2(-1, -1),
            v2(-2, 0),
            v2(-1, 1),
            v2(0, 2),
            v2(1, 1),
            v2(2, 0),
            v2(1, -1),
            v2(0, -2),
        ]
        .into_iter()
        .map(|p| snake.head() + p)
        .filter(|&p| game.grid.has(p) && !game.grid[p].owned())
        .choose(&mut rng);
        if let Some(p) = p {
            game.grid[p].set_food(true);
        }
    }

    game
}
