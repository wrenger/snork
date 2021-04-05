use std::fmt;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use super::Agent;
use crate::env::*;
use crate::game::{alphabeta, max_n, Comparable, FloodFill, Game, Outcome};

use crate::util::argmax;

use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};

/// Configuration of the tree search heuristic.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TreeConfig {
    mobility: f64,
    mobility_decay: f64,
    health: f64,
    health_decay: f64,
    len_advantage: f64,
    len_advantage_decay: f64,
    food_ownership: f64,
    food_ownership_decay: f64,
    centrality: f64,
    centrality_decay: f64,
}

impl Default for TreeConfig {
    fn default() -> Self {
        TreeConfig {
            mobility: 0.7,
            mobility_decay: 0.0,
            health: 0.012,
            health_decay: 0.0,
            len_advantage: 1.0,
            len_advantage_decay: 0.0,
            food_ownership: 0.65,
            food_ownership_decay: 0.0,
            centrality: 0.1,
            centrality_decay: 0.0,
        }
    }
}

/// Combines the different heuristics into a single value.
#[derive(PartialEq, Default, Clone, Copy)]
pub struct Evaluation(f64, f64, f64, f64, f64);

impl From<Evaluation> for f64 {
    fn from(v: Evaluation) -> f64 {
        v.0 + v.1 + v.2 + v.3 + v.4
    }
}
impl PartialOrd for Evaluation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        f64::from(*self).partial_cmp(&f64::from(*other))
    }
}
impl Comparable for Evaluation {
    fn max() -> Evaluation {
        Evaluation(std::f64::INFINITY, 0.0, 0.0, 0.0, 0.0)
    }
    fn min() -> Evaluation {
        Evaluation(-std::f64::INFINITY, 0.0, 0.0, 0.0, 0.0)
    }
}
impl fmt::Debug for Evaluation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:.2}, {:.2}, {:.2}, {:.2}, {:.2})",
            self.0, self.1, self.2, self.3, self.4
        )
    }
}

/// Tree search agent.
#[derive(Debug, Default)]
pub struct TreeAgent {
    config: TreeConfig,
}

impl TreeAgent {
    pub fn new(_request: &GameRequest, config: &TreeConfig) -> TreeAgent {
        TreeAgent {
            config: config.clone(),
        }
    }

    /// Heuristic function for the tree search.
    fn heuristic(
        food: &[Vec2D],
        flood_fill: &mut FloodFill,
        game: &Game,
        turn: usize,
        config: &TreeConfig,
    ) -> Evaluation {
        match game.outcome() {
            Outcome::Match => Evaluation::default(),
            Outcome::Winner(0) => Evaluation::max(),
            Outcome::Winner(_) => Evaluation::min(),
            Outcome::None if !game.snake_is_alive(0) => Evaluation::min(),
            Outcome::None => {
                flood_fill.flood_snakes(&game.grid, &game.snakes);
                let space = flood_fill.count_space_of(true);
                let mobility = space as f64 / (game.grid.width * game.grid.height) as f64;

                let health = game.snakes[0].health as f64 / 100.0;

                // Length advantage
                let own_len = game.snakes[0].body.len();
                let max_enemy_len = game.snakes[1..]
                    .iter()
                    .map(|s| s.body.len())
                    .max()
                    .unwrap_or(0);
                let len_advantage = own_len as f64 / max_enemy_len as f64;

                // Owned food
                let accessable_food =
                    food.iter().filter(|&&p| flood_fill[p].is_you()).count() as f64;
                let food_ownership = accessable_food / game.grid.width as f64;

                // Centrality
                let centrality = 1.0
                    - (game.snakes[0].head()
                        - Vec2D::new(game.grid.width as i16 / 2, game.grid.height as i16 / 2))
                    .manhattan() as f64
                        / game.grid.width as f64;

                Evaluation(
                    mobility * config.mobility * (-(turn as f64) * config.mobility_decay).exp(),
                    health * config.health * (-(turn as f64) * config.health_decay).exp(),
                    len_advantage
                        * config.len_advantage
                        * (-(turn as f64) * config.len_advantage_decay).exp(),
                    food_ownership
                        * config.food_ownership
                        * (-(turn as f64) * config.food_ownership_decay).exp(),
                    centrality
                        * config.centrality
                        * (-(turn as f64) * config.centrality_decay).exp(),
                )
            }
        }
    }

    /// Performes a tree search and returns the maximized heuristic and move.
    fn next_move(
        game: &Game,
        turn: usize,
        food: &[Vec2D],
        depth: usize,
        config: &TreeConfig,
    ) -> (Direction, Evaluation) {
        // Allocate and reuse flood fill memory
        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);

        let start = Instant::now();
        if game.snakes.len() == 2 {
            // Plain Alpha-Beta is faster for two agents
            let evaluation = alphabeta(&game, depth, |game| {
                TreeAgent::heuristic(&food, &mut flood_fill, game, turn + depth, config)
            });
            println!(
                "alphabeta {} {:?}ms {:?}",
                depth,
                (Instant::now() - start).as_millis(),
                evaluation
            );
            evaluation
        } else {
            // MinMax for more than two agents
            let evaluation = max_n(&game, depth, |game| {
                TreeAgent::heuristic(&food, &mut flood_fill, game, turn + depth, config)
            });
            println!(
                "max_n {} {:?}ms {:?}",
                depth,
                (Instant::now() - start).as_millis(),
                evaluation
            );
            argmax(evaluation.iter())
                .map(|d| (Direction::from(d as u8), evaluation[d]))
                .unwrap_or_default()
        }
    }

    fn iterative_tree_search(
        config: TreeConfig,
        game: Game,
        turn: usize,
        food: Vec<Vec2D>,
        start: Instant,
        duration: Duration,
        sender: Sender<Option<Direction>>,
    ) {
        // Iterative deepening
        for depth in 1..20 {
            let move_start = Instant::now();
            let (dir, value) = TreeAgent::next_move(&game, turn, &food, depth, &config);
            let move_time = Instant::now() - move_start;

            let runtime = Instant::now() - start;
            let remaining_time = if runtime < duration {
                duration - runtime
            } else {
                Duration::from_millis(0)
            };

            // Return none to fallback to random possible move
            let dir = if value > Evaluation::min() {
                Some(dir)
            } else {
                None
            };

            if sender.send(dir).is_err()
                // Terminate if we probably win/lose
                || value >= Evaluation::max()
                || value <= Evaluation::min()
                // Terminate early if the next move probably needs to much time
                || move_time * 3 * game.snakes.len() as _ > remaining_time
            {
                break;
            }
        }
    }
}

impl Agent for TreeAgent {
    fn step(&mut self, request: &GameRequest, ms: u64) -> MoveResponse {
        let start = Instant::now();
        let duration = Duration::from_millis(ms);

        let mut game = Game::new(request.board.width, request.board.height);
        game.reset_from_request(&request);

        let food = request.board.food.clone();
        let turn = request.turn;
        let config = self.config.clone();
        let game_copy = game.clone();
        let (sender, receiver) = mpsc::channel();
        // Perform tree search in a separate thread so that it can be canceled on timeout
        let handle = thread::spawn(move || {
            TreeAgent::iterative_tree_search(
                config, game_copy, turn, food, start, duration, sender,
            );
        });

        // Receive and store last result
        let mut result = None;
        while let Some(current) = receiver
            .recv_timeout({
                let runtime = Instant::now() - start;
                if runtime < duration {
                    duration - runtime
                } else {
                    Duration::from_millis(0)
                }
            })
            .ok()
            .flatten()
        {
            result = Some(current);
        }
        drop(handle);

        if let Some(dir) = result {
            println!(">>> main: {:?}", dir);
            return MoveResponse::new(dir);
        }

        println!(">>> random");

        let mut rng = SmallRng::from_entropy();
        MoveResponse::new(
            game.valid_moves(0)
                .choose(&mut rng)
                .unwrap_or(Direction::Up),
        )
    }

    fn end(&mut self, _: &GameRequest) {}
}

#[cfg(test)]
mod test {
    #[test]
    #[ignore]
    fn bench_tree() {
        use super::*;
        use std::time::Instant;

        let game_req: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
        ).unwrap();

        let mut agent = TreeAgent::default();
        const COUNT: usize = 1000;

        let start = Instant::now();
        for _ in 0..COUNT {
            let d = agent.step(&game_req, 200);
            assert_eq!(d.r#move, Direction::Down);
        }
        let end = Instant::now();
        let runtime = (end - start).as_millis();
        println!(
            "Runtime: total={}ms, avg={}ms",
            runtime,
            runtime as f64 / COUNT as f64
        )
    }
}
