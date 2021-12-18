use crate::game::search::{self, Heuristic};
use crate::game::{FloodFill, Game};

/// The new floodfill agent for royale games
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FloodHeuristic {
    board_control: f64,
    health: f64,
    len_advantage: f64,
    len_advantage_decay: f64,
    food_distance: f64,
}

impl Default for FloodHeuristic {
    fn default() -> Self {
        Self {
            board_control: 0.9,
            health: 0.045,
            len_advantage: 6.4,
            len_advantage_decay: 0.0,
            food_distance: 0.415,
        }
    }
}

impl Heuristic for FloodHeuristic {
    fn eval(&self, game: &Game) -> f64 {
        if game.snake_is_alive(0) {
            let own_len = game.snakes[0].body.len() as f64;
            let area = (game.grid.width * game.grid.height) as f64;

            let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
            let food_distances = flood_fill.flood_snakes(&game.grid, &game.snakes);

            let board_control = (flood_fill.count_health(0) as f64 / (area * 100.0)).sqrt();

            let food_distance = food_distances
                .into_iter()
                .filter(|&d| d < u16::MAX)
                .map(|d| (area - d as f64) / area)
                .sum::<f64>();

            let health = (game.snakes[0].health as f64 / 100.0).sqrt();

            let max_enemy_len = game.snakes[1..]
                .iter()
                .filter(|s| s.alive())
                .map(|s| s.body.len())
                .max()
                .unwrap_or(1)
                .max(1) as f64;

            // Sqrt because if we are larger we do not have to as grow much anymore.
            let len_advantage =
                ((own_len + food_distance * self.food_distance) / max_enemy_len).sqrt();

            self.board_control * board_control
                + self.health * health
                + self.len_advantage
                    * len_advantage
                    * (-(game.turn as f64) * self.len_advantage_decay).exp()
        } else {
            search::LOSS
        }
    }
}
