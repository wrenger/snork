use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use crate::env::*;
use crate::game::Game;
use crate::search::{self, Heuristic};

use crate::util::argmax;

use log::info;
use tokio::sync::mpsc;
use tokio::time;

const FAST_TIMEOUT: u64 = 150;
const MAX_DEPTH: usize = 16;

pub async fn step(heuristic: Arc<dyn Heuristic>, timeout: u64, game: &Game) -> MoveResponse {
    if timeout <= FAST_TIMEOUT {
        return step_fast(&*heuristic, game);
    }

    let (sender, mut receiver) = mpsc::channel(MAX_DEPTH);

    let _ = time::timeout(
        Duration::from_millis(timeout),
        iterative_tree_search(heuristic, game, sender),
    )
    .await;

    let mut result = None;
    while let Some(dir) = receiver.recv().await {
        result = Some(dir);
    }

    if let Some(dir) = result {
        return MoveResponse::new(Direction::from(dir as u8));
    }

    info!(">>> none");
    MoveResponse::new(game.valid_moves(0).next().unwrap_or(Direction::Up))
}

pub fn step_fast(heuristic: &dyn Heuristic, game: &Game) -> MoveResponse {
    let start = Instant::now();
    let result = search::max_n(game, 1, heuristic);

    info!(">>> max_n 1 {:?}ms {result:?}", start.elapsed().as_millis());

    if let Some(dir) = argmax(result.iter().copied()) {
        if result[dir] > search::LOSS {
            return MoveResponse::new(Direction::from(dir as u8));
        }
    }

    info!(">>> none");
    MoveResponse::new(game.valid_moves(0).next().unwrap_or(Direction::Up))
}

async fn iterative_tree_search(
    heuristic: Arc<dyn Heuristic>,
    game: &Game,
    sender: mpsc::Sender<Direction>,
) {
    // Iterative deepening
    for depth in 1..MAX_DEPTH {
        let heuristic = heuristic.clone();
        let (dir, value) = tree_search(heuristic, game, depth).await;

        // Stop and fallback to random possible move
        if value <= search::LOSS {
            break;
        }

        if sender.send(dir).await.is_err()
            // Terminate if we probably win/lose
            || value >= search::WIN
        {
            break;
        }
    }
}

/// Performes a tree search and returns the maximized heuristic and move.
pub async fn tree_search(
    heuristic: Arc<dyn Heuristic>,
    game: &Game,
    depth: usize,
) -> (Direction, f64) {
    let start = Instant::now();

    let result = search::async_max_n(game, depth, heuristic).await;

    info!(
        ">>> max_n {depth} {:?}ms {result:.3?}",
        start.elapsed().as_millis(),
    );

    argmax(result.iter().copied())
        .map(|d| (Direction::from(d as u8), result[d]))
        .unwrap()
}
