use std::fmt;
use std::time::Duration;
use std::time::Instant;

use crate::env::*;
use crate::game::{async_alphabeta, async_max_n, Comparable, FloodFill, Game, Outcome};

use crate::util::argmax;

use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

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
    pub fn heuristic(game: &Game, turn: usize, config: &TreeConfig) -> Evaluation {
        match game.outcome() {
            Outcome::Match => Evaluation::default(),
            Outcome::Winner(0) => Evaluation::max(),
            Outcome::Winner(_) => Evaluation::min(),
            Outcome::None if !game.snake_is_alive(0) => Evaluation::min(),
            Outcome::None => {
                let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
                let food_distances = flood_fill.flood_snakes(&game.grid, &game.snakes);
                let space = flood_fill.count_space(true);
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
                    food_distances.into_iter().filter(|&p| p < u16::MAX).count() as f64;
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
    pub async fn next_move(
        game: &Game,
        turn: usize,
        depth: usize,
        config: &TreeConfig,
    ) -> (Direction, Evaluation) {
        // Allocate and reuse flood fill memory
        let start = Instant::now();
        if game.snakes.len() == 2 {
            // Alpha-Beta is faster for two agents
            let config = config.clone();
            let evaluation = async_alphabeta(&game, depth, move |game| {
                TreeAgent::heuristic(game, turn + depth, &config)
            })
            .await;

            println!(
                "alphabeta {} {:?}ms {:?}",
                depth,
                (Instant::now() - start).as_millis(),
                evaluation
            );
            evaluation
        } else {
            // MinMax for more than two agents
            let config = config.clone();
            let evaluation = async_max_n(&game, depth, move |game| {
                TreeAgent::heuristic(game, turn + depth, &config)
            })
            .await;

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

    async fn iterative_tree_search(
        config: TreeConfig,
        game: Game,
        turn: usize,
        sender: Sender<Direction>,
    ) {
        // Iterative deepening
        for depth in 1..20 {
            let (dir, value) = TreeAgent::next_move(&game, turn, depth, &config).await;

            // Stop and fallback to random possible move
            if value <= Evaluation::min() {
                break;
            };

            if sender.send(dir).await.is_err()
                // Terminate if we probably win/lose
                || value >= Evaluation::max()
                || value <= Evaluation::min()
            {
                break;
            }
        }
    }

    pub async fn step(&mut self, request: &GameRequest, ms: u64) -> MoveResponse {
        let mut game = Game::new(request.board.width, request.board.height);
        game.reset_from_request(&request);

        let turn = request.turn;
        let config = self.config.clone();
        let game_copy = game.clone();
        let (sender, mut receiver) = mpsc::channel(32);

        let _ = tokio::time::timeout(
            Duration::from_millis(ms),
            TreeAgent::iterative_tree_search(config, game_copy, turn, sender),
        )
        .await;

        // Receive and store last result
        let mut result = None;
        while let Some(current) = receiver.recv().await {
            result = Some(current);
        }

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
}
