use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Instant;

use log::{warn, info};

use crate::env::*;
use crate::game::search::Heuristic;
use crate::game::{search, FloodFill, Game, Snake};
use crate::util::{argmax, OrdPair};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct MobilityAgent {
    /// [0, 100]
    health_threshold: u8,
    /// [0, width * height]
    min_len: usize,
    /// [0, 3]
    first_move_cost: f64,
}

impl Default for MobilityAgent {
    fn default() -> MobilityAgent {
        MobilityAgent {
            health_threshold: 35,
            min_len: 8,
            first_move_cost: 1.0,
        }
    }
}

/// Simple space after move heuristic
#[derive(Debug, Clone, Default)]
struct MobilityHeuristic;

impl Heuristic for MobilityHeuristic {
    fn eval(&self, game: &Game) -> f64 {
        if game.snake_is_alive(0) {
            let mut flood_fill = FloodFill::new(game.grid.width, game.grid.width);
            flood_fill.flood_snakes(&game.grid, &game.snakes);
            flood_fill.count_space(true) as f64
        } else {
            0.0
        }
    }
}

impl MobilityAgent {
    fn find_food(
        &self,
        game: &Game,
        food: &[Vec2D],
        flood_fill: &FloodFill,
        space_after_move: &[f64; 4],
    ) -> Option<Direction> {
        let you: &Snake = &game.snakes[0];

        let area = (game.grid.width * game.grid.height) as f64;

        // Heuristic for preferring high movement
        let first_move_costs = space_after_move.map(|x| (1.0 - x / area) * self.first_move_cost);

        // Avoid longer enemy heads
        let mut grid = game.grid.clone();
        for snake in &game.snakes[1..] {
            if snake.body.len() >= you.body.len() {
                for d in Direction::iter() {
                    let p = snake.head().apply(d);
                    if grid.has(p) {
                        grid[p].set_owned(true);
                    }
                }
            }
        }

        let mut food_dirs = BinaryHeap::new();
        for &p in food {
            if let Some(path) = grid.a_star(you.head(), p, &first_move_costs) {
                if path.len() >= 2 {
                    let costs = path.len() + if flood_fill[p].is_you() { 0 } else { 5 };
                    food_dirs.push(OrdPair(Reverse(costs), Direction::from(path[1] - path[0])));
                }
            }
        }

        while let Some(OrdPair(_, dir)) = food_dirs.pop() {
            // Is there enough space for us?
            if space_after_move[dir as u8 as usize] as usize >= you.body.len() - 1 {
                return Some(dir);
            }
        }
        None
    }

    pub async fn step(&self, request: &GameRequest, _: u64) -> MoveResponse {
        let game = Game::from_request(request);
        let you = &game.snakes[0];

        // Flood fill heuristics
        let start = Instant::now();
        let space_after_move = search::max_n(&game, 1, &MobilityHeuristic);
        info!(
            "max_n {:?}ms {:?}",
            start.elapsed().as_millis(),
            space_after_move
        );

        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
        flood_fill.flood_snakes(&game.grid, &game.snakes);

        // Find Food
        if you.body.len() < self.min_len || you.health < self.health_threshold {
            if let Some(dir) =
                self.find_food(&game, &request.board.food, &flood_fill, &space_after_move)
            {
                info!(">>> find food");
                return MoveResponse::new(dir);
            }
        }

        // Maximize mobility
        if let Some(dir) = argmax(space_after_move.iter()) {
            if space_after_move[dir] > 0.0 {
                info!(">>> max space");
                return MoveResponse::new(Direction::from(dir as u8));
            }
        }

        warn!(">>> random");
        MoveResponse::new(game.valid_moves(0).next().unwrap_or(Direction::Up))
    }
}
