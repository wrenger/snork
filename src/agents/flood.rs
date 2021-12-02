use rand::prelude::*;
use rand::SeedableRng;

use super::Agent;
use crate::env::*;
use crate::game::{max_n, FloodFill, Game};
use crate::util::argmax;

#[derive(Debug)]
pub struct FloodAgent {
    game: Game,
    flood_fill: FloodFill,
    config: FloodConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FloodConfig {
    board_control: f64,
    health: f64,
    len_advantage: f64,
    food_distance: f64,
}

impl Default for FloodConfig {
    fn default() -> Self {
        Self {
            board_control: 1.0,
            health: 0.5,
            len_advantage: 1.0,
            food_distance: 1.0,
        }
    }
}

impl FloodAgent {
    pub fn new(request: &GameRequest, config: &FloodConfig) -> Self {
        Self {
            game: Game::new(request.board.width, request.board.width),
            flood_fill: FloodFill::new(request.board.width, request.board.width),
            config: config.clone(),
        }
    }
}

impl Agent for FloodAgent {
    fn step(&mut self, request: &GameRequest, _ms: u64) -> MoveResponse {
        self.game.reset_from_request(&request);

        let flood_fill = &mut self.flood_fill;
        let config = &self.config;

        let area = (self.game.grid.width * self.game.grid.height) as f64;

        let space_after_move = max_n(&self.game, 1, |game| {
            if game.snake_is_alive(0) {
                let own_len = game.snakes[0].body.len() as f64;

                let food_distances = flood_fill.flood_snakes(&game.grid, &game.snakes);
                let mut food_distance = food_distances
                    .into_iter()
                    .map(|d| (area - d as f64) / area)
                    .sum::<f64>();

                food_distance = (food_distance + own_len) / area;

                let board_control = flood_fill.count_health(true) as f64 / (area * 100.0);

                let health = game.snakes[0].health as f64 / 100.0;

                let max_enemy_len = game.snakes[1..]
                    .iter()
                    .map(|s| s.body.len())
                    .max()
                    .unwrap_or(0) as f64;
                let len_advantage = (own_len / max_enemy_len).min(2.0);

                config.board_control * board_control
                    + config.health * health
                    + config.len_advantage * len_advantage
                    + config.food_distance * food_distance
            } else {
                0.0
            }
        });

        println!(">>> space {:?}", space_after_move);

        if let Some(dir) = argmax(space_after_move.iter()) {
            if space_after_move[dir] > 0.0 {
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
