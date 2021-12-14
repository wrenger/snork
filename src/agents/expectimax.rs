use std::time::Duration;
use std::time::Instant;

use crate::env::*;
use crate::game::search::{self, Heuristic};
use crate::game::Game;

use log::info;
use tokio::sync::mpsc;
use tokio::time;

const FAST_TIMEOUT: u64 = 150;
const MAX_DEPTH: usize = 8;

pub async fn step<H: Heuristic>(
    heuristic: &H,
    request: &GameRequest,
    latency: u64,
) -> MoveResponse {
    let ms = request.game.timeout.saturating_sub(latency);
    if ms <= FAST_TIMEOUT {
        return super::maxn::step_fast(heuristic, request);
    }

    let game = Game::from_request(request);

    let (sender, mut receiver) = mpsc::channel(MAX_DEPTH);

    let _ = time::timeout(
        Duration::from_millis(ms),
        iterative_tree_search(heuristic, &game, sender),
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

/// TODO: This agent is not fully implemented yet!
async fn iterative_tree_search<H: Heuristic>(
    heuristic: &H,
    game: &Game,
    sender: mpsc::Sender<Direction>,
) {
    // Iterative deepening
    for depth in 1..MAX_DEPTH {
        let start = Instant::now();
        let (dir, value) = search::expectimax(game, depth, heuristic).await;

        info!(
            ">>> expectimax {} {:?}ms {:?} {:?}",
            depth,
            start.elapsed().as_millis(),
            dir,
            value
        );

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
