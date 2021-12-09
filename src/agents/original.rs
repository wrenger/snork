use std::fmt;

use crate::env::*;
use crate::game::{Comparable, FloodFill, Game};

use super::tree::Heuristic;

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

impl Heuristic for TreeHeuristic {
    type Eval = Evaluation;
    /// Heuristic function for the tree search.
    fn heuristic(&self, game: &Game, turn: usize) -> Evaluation {
        if !game.snake_is_alive(0) {
            return Evaluation::min();
        }

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
        let accessable_food = food_distances.into_iter().filter(|&p| p < u16::MAX).count() as f64;
        let food_ownership = accessable_food / game.grid.width as f64;

        // Centrality
        let centrality = 1.0
            - (game.snakes[0].head()
                - Vec2D::new(game.grid.width as i16 / 2, game.grid.height as i16 / 2))
            .manhattan() as f64
                / game.grid.width as f64;

        Evaluation(
            mobility * self.mobility * (-(turn as f64) * self.mobility_decay).exp(),
            health * self.health * (-(turn as f64) * self.health_decay).exp(),
            len_advantage * self.len_advantage * (-(turn as f64) * self.len_advantage_decay).exp(),
            food_ownership
                * self.food_ownership
                * (-(turn as f64) * self.food_ownership_decay).exp(),
            centrality * self.centrality * (-(turn as f64) * self.centrality_decay).exp(),
        )
    }
}