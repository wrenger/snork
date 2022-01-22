use crate::env::*;
use crate::game::search::{self, Heuristic};
use crate::game::{FloodFill, Game};

/// Configuration of the tree search heuristic.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TreeHeuristic {
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

impl Default for TreeHeuristic {
    fn default() -> Self {
        Self {
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

impl Heuristic for TreeHeuristic {
    /// Heuristic function for the tree search.
    fn eval(&self, game: &Game) -> f64 {
        if !game.snake_is_alive(0) {
            return search::LOSS;
        }

        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
        let food_distances = flood_fill.flood_snakes(&game.grid, &game.snakes);
        let space = flood_fill.count_space(0);
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
        let accessable_food = food_distances.into_iter().count() as f64;
        let food_ownership = accessable_food / game.grid.width as f64;

        // Centrality
        let centrality = 1.0
            - (game.snakes[0].head()
                - Vec2D::new(game.grid.width as i16 / 2, game.grid.height as i16 / 2))
            .manhattan() as f64
                / game.grid.width as f64;

        mobility * self.mobility * (-(game.turn as f64) * self.mobility_decay).exp()
            + health * self.health * (-(game.turn as f64) * self.health_decay).exp()
            + len_advantage
                * self.len_advantage
                * (-(game.turn as f64) * self.len_advantage_decay).exp()
            + food_ownership
                * self.food_ownership
                * (-(game.turn as f64) * self.food_ownership_decay).exp()
            + centrality * self.centrality * (-(game.turn as f64) * self.centrality_decay).exp()
    }
}
