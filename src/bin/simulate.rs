use clap::Parser;
use log::{debug, info, warn};
use owo_colors::OwoColorize;

use snork::agents::Agent;
use snork::env::*;
use snork::game::{Game, Outcome, Snake};
use snork::grid::CellT;
use snork::logging;

use rand::prelude::*;
use rand::seq::IteratorRandom;
use std::iter::repeat;
use std::time::Instant;

#[derive(clap::Parser)]
#[clap(version, author, about = "Simulate a game between agents.")]
struct Opts {
    /// Time each snake has for a turn.
    #[clap(long, default_value_t = 200)]
    timeout: u64,
    /// Board height.
    #[clap(long, default_value_t = 11)]
    width: usize,
    /// Board width.
    #[clap(long, default_value_t = 11)]
    height: usize,
    /// Chance new food spawns.
    #[clap(long, default_value_t = 0.15)]
    food_rate: f64,
    /// Number of turns after which the hazard expands.
    #[clap(short, long, default_value_t = 25)]
    shrink_turns: usize,
    /// Number of games that are played.
    #[clap(short, long, default_value_t = 1)]
    game_count: usize,
    /// Swap agent positions to get more accurate results.
    #[clap(long)]
    swap: bool,
    /// Seed for the random number generator.
    #[clap(long, default_value_t = 0)]
    seed: u64,
    /// Start config.
    #[clap(long, value_parser = parse_request)]
    init: Option<GameRequest>,
    /// Configurations.
    #[clap()]
    agents: Vec<Agent>,
}

fn parse_request(s: &str) -> Result<GameRequest, serde_json::Error> {
    serde_json::from_str(s)
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
        swap,
        seed,
        init,
        mut agents,
    } = Opts::parse();

    assert!(agents.len() <= 4, "Only up to 4 snakes are supported");
    info!("agents: {agents:?}");

    let start = Instant::now();

    let mut wins = repeat(0).take(agents.len()).collect::<Vec<usize>>();

    for _ in 0..agents.len() {
        let mut rng = if seed == 0 {
            SmallRng::from_entropy()
        } else {
            SmallRng::seed_from_u64(seed)
        };

        for i in 0..game_count {
            let mut game = if let Some(request) = &init {
                Game::from_request(request)
            } else {
                init_game(width, height, agents.len(), &mut rng)
            };

            let outcome = play_game(
                &agents,
                &mut game,
                timeout,
                food_rate,
                shrink_turns,
                &mut rng,
            )
            .await;
            if let Outcome::Winner(winner) = outcome {
                wins[winner as usize] += 1;
            }
            warn!(
                "{}: {i} {}ms",
                "Finish Game".bright_green(),
                start.elapsed().as_millis()
            );
        }

        if !swap {
            break;
        }
        // Swap agents
        wins.rotate_left(1);
        agents.rotate_left(1);
    }

    println!("Agents: {agents:?}");
    println!("Result: {wins:?}");
}

async fn play_game(
    agents: &[Agent],
    game: &mut Game,
    timeout: u64,
    food_rate: f64,
    shrink_turns: usize,
    rng: &mut SmallRng,
) -> Outcome {
    let mut food_count = 4;

    debug!("init: {game:?}");

    let mut hazard_insets = [0; 4];

    for turn in game.turn.. {
        let mut moves = [Direction::Up; 4];
        for i in 0..game.snakes.len() {
            if game.snakes[i].alive() {
                // Agents assume player 0 is you.
                game.snakes.swap(0, i);

                let response = agents[i].step_internal(timeout, game).await;
                moves[i] = response.r#move;

                game.snakes.swap(0, i);
            }
        }
        debug!("Moves: {moves:?}");

        game.step(&moves);

        debug!("{}: {:?}", turn, game);

        let outcome = game.outcome();
        if outcome != Outcome::None {
            warn!("game: {outcome:?} after {turn} turns");
            return outcome;
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
                .filter(|c| c.t == CellT::Free)
                .choose(rng)
            {
                cell.t = CellT::Food;
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
                    game.grid[v2(x as _, y as _)].hazard = true;
                }
            } else {
                let x = if dir == 1 {
                    hazard_insets[dir] - 1
                } else {
                    game.grid.width - hazard_insets[dir]
                };
                for y in 0..game.grid.height {
                    game.grid[v2(x as _, y as _)].hazard = true;
                }
            }
        }
    }
    Outcome::Match
}

fn init_game(width: usize, height: usize, num_agents: usize, rng: &mut SmallRng) -> Game {
    if width % 2 == 0 || height % 2 == 0 {
        warn!("If the dimension are even, the initial board configuration is unfair!");
    }
    if width != height {
        warn!("If width != height, the initial board configuration is unfair!");
    }

    // Either start in the corners or in the middle of the edges
    let mut start_positions = if rng.gen() {
        // Corners
        [
            v2(1, 1),
            v2((width - 2) as _, 1),
            v2((width - 2) as _, (height - 2) as _),
            v2(1, (height - 2) as _),
        ]
    } else {
        // Edges
        [
            v2((width / 2) as _, 1),
            v2((width - 2) as _, (height / 2) as _),
            v2((width / 2) as _, (height - 2) as _),
            v2(1, (height / 2) as _),
        ]
    }
    .into_iter()
    .choose_multiple(rng, num_agents);

    start_positions.shuffle(rng);

    let snakes = start_positions
        .into_iter()
        .map(|p| Snake::new(vec![p; 3].into(), 100))
        .collect();

    let mut game = Game::new(0, width, height, snakes, &[], &[]);

    // Food at center
    game.grid[(width / 2, height / 2).into()].t = CellT::Food;

    // Spawn 1 food 2 steps away from each snake
    for snake in game.snakes.clone() {
        let p = [v2(-1, -1), v2(-1, 1), v2(1, 1), v2(1, -1)]
            .into_iter()
            .map(|p| snake.head() + p)
            // Only free cells on the board
            .filter(|&p| game.grid.has(p) && game.grid[p].t != CellT::Owned)
            // Limit to a border cells (excluding the corners)
            .filter(|&p| {
                (p.x == 0 || p.x == game.grid.width as i16 - 1)
                    ^ (p.y == 0 || p.y == game.grid.height as i16 - 1)
            })
            .choose(rng);
        if let Some(p) = p {
            game.grid[p].t = CellT::Food;
        }
    }

    game
}
