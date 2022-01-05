use crate::game::search::{self, Heuristic};
use crate::game::{FloodFill, Game};

/// The new floodfill agent for royale games
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SoloHeuristic {
    saturated: f64,
    space: f64,
    size: f64,
}

impl Default for SoloHeuristic {
    fn default() -> Self {
        Self {
            saturated: 0.1,
            space: 1.0,
            size: 0.5,
        }
    }
}

impl Heuristic for SoloHeuristic {
    fn eval(&self, game: &Game) -> f64 {
        if game.snake_is_alive(0) {
            let you = &game.snakes[0];
            let area = (game.grid.width * game.grid.height) as f64;

            let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
            let food_distances = flood_fill.flood_snakes(&game.grid, &game.snakes);

            let food_distance = food_distances[0] as f64;
            let saturated = if food_distance < you.health as f64 {
                1.0
            } else {
                0.0
            };

            let space = flood_fill.count_space(0) as f64 / area;
            let size = (3.0 / you.body.len() as f64).sqrt();

            self.saturated * saturated + self.space * space + self.size * size
        } else {
            search::LOSS
        }
    }
}
