use crate::game::{FloodFill, Game};

use super::tree::Heuristic;

/// The new floodfill agent for royale games
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FloodHeuristic {
    board_control: f64,
    health: f64,
    len_advantage: f64,
    food_distance: f64,
}

impl Default for FloodHeuristic {
    fn default() -> Self {
        Self {
            board_control: 2.0,
            health: 0.5,
            len_advantage: 8.0,
            food_distance: 0.5,
        }
    }
}

impl Heuristic for FloodHeuristic {
    type Eval = f64;

    fn heuristic(&self, game: &Game, _: usize) -> f64 {
        if game.snake_is_alive(0) {
            let own_len = game.snakes[0].body.len() as f64;
            let area = (game.grid.width * game.grid.height) as f64;

            let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
            let food_distances = flood_fill.flood_snakes(&game.grid, &game.snakes);
            let food_distance = food_distances
                .into_iter()
                .filter(|&d| d < u16::MAX)
                .map(|d| (area - d as f64) / area)
                .sum::<f64>();

            let board_control = flood_fill.count_health(true) as f64 / (area * 100.0);

            let health = game.snakes[0].health as f64 / 100.0;

            let max_enemy_len = game.snakes[1..]
                .iter()
                .map(|s| s.body.len())
                .max()
                .unwrap_or(0) as f64;
            // Sqrt because if we are larger we do not have to as grow much anymore.
            let len_advantage =
                ((own_len + food_distance * self.food_distance) / max_enemy_len).sqrt();

            self.board_control * board_control
                + self.health * health
                + self.len_advantage * len_advantage
        } else {
            -f64::INFINITY
        }
    }
}
