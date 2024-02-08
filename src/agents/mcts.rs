use std::sync::Arc;

use crate::env::MoveResponse;
use crate::game::Game;
use crate::search::{mcts, Heuristic};

pub async fn step(heuristic: Arc<dyn Heuristic>, timeout: u64, game: &Game) -> MoveResponse {
    let dir = mcts(heuristic, timeout, game).await;
    MoveResponse::new(dir)
}
