use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Instant;

use rand::seq::IteratorRandom;
use rand::{rngs::SmallRng, SeedableRng};

use super::Agent;
use crate::env::*;
use crate::game::{max_n, FloodFill, Game, Grid, Snake};
use crate::util::{argmax, OrdPair};

#[derive(Debug)]
pub struct MobilityAgent {
    game: Game,
    flood_fill: FloodFill,
    config: MobilityConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct MobilityConfig {
    /// [0, 100]
    health_threshold: u8,
    /// [0, width * height]
    min_len: usize,
    /// [0, 3]
    first_move_cost: f64,
}

impl Default for MobilityConfig {
    fn default() -> MobilityConfig {
        MobilityConfig {
            health_threshold: 35,
            min_len: 10,
            first_move_cost: 1.0,
        }
    }
}

impl MobilityAgent {
    pub fn new(request: &GameRequest, config: &MobilityConfig) -> MobilityAgent {
        MobilityAgent {
            game: Game::new(request.board.width, request.board.width),
            flood_fill: FloodFill::new(request.board.width, request.board.width),
            config: config.clone(),
        }
    }

    fn find_food(
        &self,
        food: &[Vec2D],
        grid: &Grid,
        space_after_move: &[f64; 4],
    ) -> Option<Direction> {
        let you: &Snake = &self.game.snakes[0];

        // Heuristic for preferring high movement
        let first_move_costs = [
            (1.0 - space_after_move[0] / (grid.width * grid.height) as f64)
                * self.config.first_move_cost,
            (1.0 - space_after_move[1] / (grid.width * grid.height) as f64)
                * self.config.first_move_cost,
            (1.0 - space_after_move[2] / (grid.width * grid.height) as f64)
                * self.config.first_move_cost,
            (1.0 - space_after_move[3] / (grid.width * grid.height) as f64)
                * self.config.first_move_cost,
        ];

        let mut food_dirs = BinaryHeap::new();
        for &p in food {
            if let Some(path) = grid.a_star(you.head(), p, &first_move_costs) {
                if path.len() >= 2 {
                    let costs = path.len() + if self.flood_fill[p].is_you() { 0 } else { 5 };
                    food_dirs.push(OrdPair(Reverse(costs), Direction::from(path[1] - path[0])));
                }
            }
        }

        while let Some(OrdPair(_, dir)) = food_dirs.pop() {
            if space_after_move[dir as u8 as usize] as usize >= you.body.len() - 1 {
                return Some(dir);
            }
        }
        None
    }
}

impl Agent for MobilityAgent {
    fn step(&mut self, request: &GameRequest, _: u64) -> MoveResponse {
        self.game.reset_from_request(&request);
        let you = &self.game.snakes[0];

        // Flood fill heuristics
        let start = Instant::now();
        let flood_fill = &mut self.flood_fill;
        let space_after_move = max_n(&self.game, 1, |game| {
            if game.snake_is_alive(0) {
                flood_fill.flood_snakes(&game.grid, &game.snakes);
                flood_fill.count_space(true) as f64
            } else {
                0.0
            }
        });
        println!("max_n {:?}ms", (Instant::now() - start).as_millis());
        self.flood_fill
            .flood_snakes(&self.game.grid, &self.game.snakes);

        // Avoid longer enemy heads
        let mut grid = self.game.grid.clone();
        for snake in &self.game.snakes[1..] {
            if snake.body.len() >= you.body.len() {
                for d in Direction::iter() {
                    let p = snake.head().apply(d);
                    if grid.has(p) {
                        grid[p].set_owned(true);
                    }
                }
            }
        }

        // Find Food
        if you.body.len() < self.config.min_len || you.health < self.config.health_threshold {
            if let Some(dir) = self.find_food(&request.board.food, &grid, &space_after_move) {
                println!(">>> find food");
                return MoveResponse::new(dir);
            }
        }

        if let Some(dir) = argmax(space_after_move.iter()) {
            if space_after_move[dir] > 0.0 {
                println!(">>> max space");
                return MoveResponse::new(Direction::from(dir as u8));
            }
        }

        println!(">>> random");
        let mut rng = SmallRng::from_entropy();
        MoveResponse::new(
            self.game
                .valid_moves(0)
                .choose(&mut rng)
                .unwrap_or(Direction::Up),
        )
    }

    fn end(&mut self, _: &GameRequest) {}
}
