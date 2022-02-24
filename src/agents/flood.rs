use crate::floodfill::FloodFill;
use crate::game::Game;
use crate::search::{self, Heuristic};

/// The new floodfill agent for royale games
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FloodHeuristic {
    health: f64,
    food_distance: f64,
    space: f64,
    space_adv: f64,
    size_adv: f64,
    size_adv_decay: f64,
}

impl Default for FloodHeuristic {
    fn default() -> Self {
        Self {
            health: 0.00044,
            food_distance: 0.173,
            space: 0.0026,
            space_adv: 0.108,
            size_adv: 7.049,
            size_adv_decay: 0.041,
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

            // Health is more important if we have not much
            let health = (game.snakes[0].health as f64 / 100.0).sqrt();

            // Space advantage becomes increasingly better when higher
            let space = flood_fill.count_health(0) as f64;

            let (size_adv, space_adv) = if let Some((i, longest_enemy)) = game
                .snakes
                .iter()
                .enumerate()
                .skip(1)
                .filter(|(_, s)| s.alive())
                .max_by_key(|(_, s)| s.body.len())
            {
                // Distance to the nearest four food cells
                let food_distance = food_distances
                    .iter()
                    .map(|&d| (area - d as f64) / area)
                    .sum::<f64>();
                let enemy_len = longest_enemy.body.len() as f64;
                // Sqrt because if we are larger we do not have to as grow much anymore.
                let size_adv = ((own_len + food_distance * self.food_distance) / enemy_len).sqrt();

                let enemy_space = flood_fill.count_health(i as _) as f64;
                let space_adv = if space > 0.0 {
                    // x^3 so that the effect is stronger when the value is higher.
                    (space / (enemy_space + space)).powi(3)
                } else {
                    0.0
                };
                (size_adv, space_adv)
            } else {
                (0.0, 0.0)
            };

            let space = (space / (area * 100.0)).sqrt();

            self.health * health
                + self.space_adv * space_adv
                + self.space * space
                + self.size_adv * size_adv * (-(game.turn as f64) * self.size_adv_decay).exp2()
        } else {
            search::LOSS
        }
    }
}
