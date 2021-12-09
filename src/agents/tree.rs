use std::fmt::Debug;
use std::time::Duration;
use std::time::Instant;

use crate::env::*;
use crate::game::max_n;
use crate::game::{async_max_n, Comparable, Game};

use crate::util::argmax;

use tokio::sync::mpsc;
use tokio::time;

const FAST_TIMEOUT: u64 = 200;
const MAX_DEPTH: usize = 8;

pub trait Heuristic: Default + Debug + Clone + Send + Sync + 'static {
    type Eval: Comparable;
    fn heuristic(&self, game: &Game, turn: usize) -> Self::Eval;
}

pub async fn step<H: Heuristic>(config: &H, request: &GameRequest, latency: u64) -> MoveResponse {
    let ms = request.game.timeout.saturating_sub(latency);
    if ms <= FAST_TIMEOUT {
        return step_fast(config, request);
    }

    let game = Game::from_request(request);

    let (sender, mut receiver) = mpsc::channel(MAX_DEPTH);

    let _ = time::timeout(
        Duration::from_millis(ms),
        iterative_tree_search(config, &game, request.turn, sender),
    )
    .await;

    let mut result = None;
    while let Some(dir) = receiver.recv().await {
        result = Some(dir);
    }

    if let Some(dir) = result {
        return MoveResponse::new(Direction::from(dir as u8));
    }

    println!(">>> none");
    MoveResponse::new(game.valid_moves(0).next().unwrap_or(Direction::Up))
}

pub fn step_fast<H: Heuristic>(config: &H, request: &GameRequest) -> MoveResponse {
    let game = Game::from_request(request);

    let start = Instant::now();
    let result = max_n(&game, 1, |game| config.heuristic(game, request.turn));

    println!(
        ">>> max_n 1 {:?}ms {:?}",
        start.elapsed().as_millis(),
        result
    );

    if let Some(dir) = argmax(result.iter()) {
        if result[dir] > H::Eval::min() {
            return MoveResponse::new(Direction::from(dir as u8));
        }
    }

    println!(">>> none");
    MoveResponse::new(game.valid_moves(0).next().unwrap_or(Direction::Up))
}

async fn iterative_tree_search<H: Heuristic>(
    config: &H,
    game: &Game,
    turn: usize,
    sender: mpsc::Sender<Direction>,
) {
    // Iterative deepening
    for depth in 1..MAX_DEPTH {
        let (dir, value) = tree_search(config, game, turn, depth).await;

        // Stop and fallback to random possible move
        if value <= H::Eval::min() {
            break;
        }

        if sender.send(dir).await.is_err()
            // Terminate if we probably win/lose
            || value >= H::Eval::max()
        {
            break;
        }
    }
}

/// Performes a tree search and returns the maximized heuristic and move.
pub async fn tree_search<H: Heuristic>(
    config: &H,
    game: &Game,
    turn: usize,
    depth: usize,
) -> (Direction, H::Eval) {
    let start = Instant::now();

    let config = config.clone();
    let result = async_max_n(&game, depth, move |game| {
        config.heuristic(game, turn + depth)
    })
    .await;

    println!(
        ">>> max_n {} {:?}ms {:?}",
        depth,
        start.elapsed().as_millis(),
        result
    );

    argmax(result.iter())
        .map(|d| (Direction::from(d as u8), result[d]))
        .unwrap_or_default()
}
