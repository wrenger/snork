mod minimax;
pub use minimax::*;
mod alphabeta;
pub use alphabeta::*;
mod mcts;
pub use mcts::*;

use std::fmt::Debug;

use crate::game::Game;

pub const WIN: f64 = 10000.0;
pub const DRAW: f64 = 0.0;
pub const LOSS: f64 = -10000.0;

/// A heuristic that evaluates the game state at the leafs of a tree search.
pub trait Heuristic: Debug + Send + Sync + 'static {
    fn eval(&self, game: &Game) -> f64;
}
