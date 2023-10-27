use std::sync::Arc;

use super::{Heuristic, DRAW, LOSS, WIN};
use crate::env::*;
use crate::game::{Game, Outcome};

use async_recursion::async_recursion;

pub async fn async_alphabeta(
    game: &Game,
    depth: usize,
    heuristic: Arc<dyn Heuristic>,
) -> (Direction, f64) {
    async_alphabeta_rec(game, [Direction::Up; 4], depth, 0, LOSS, WIN, heuristic).await
}

/// # WARNING
/// This version is very slow, even slower than the synchronous alphabeta
/// and much slower than multithreaded max n
#[async_recursion]
async fn async_alphabeta_rec(
    game: &Game,
    actions: [Direction; 4],
    depth: usize,
    ply: usize,
    mut alpha: f64,
    mut beta: f64,
    heuristic: Arc<dyn Heuristic>,
) -> (Direction, f64) {
    if ply == game.snakes.len() {
        let mut game = game.clone();
        game.step(&actions);
        match game.outcome() {
            Outcome::Winner(0) => return (Direction::Up, WIN),
            Outcome::Winner(_) => return (Direction::Up, LOSS),
            Outcome::Match => return (Direction::Up, DRAW),
            Outcome::None => {}
        }

        if depth == 0 {
            (Direction::Up, heuristic.eval(&game))
        } else {
            async_alphabeta_rec(
                &game,
                [Direction::Up; 4],
                depth - 1,
                0,
                alpha,
                beta,
                heuristic,
            )
            .await
        }
    } else if ply == 0 {
        let mut value = (Direction::Up, LOSS);

        let mut futures = [None, None, None, None];
        for d in Direction::iter() {
            let game = game.clone();
            let heuristic = heuristic.clone();
            let actions = [d, Direction::Up, Direction::Up, Direction::Up];
            futures[d as u8 as usize] = Some(tokio::task::spawn(async move {
                async_alphabeta_rec(&game, actions, depth, ply + 1, alpha, beta, heuristic).await
            }));
        }

        for (i, future) in futures.into_iter().enumerate() {
            if let Some(future) = future {
                if let Ok(newval) = future.await {
                    if newval.1 > value.1 {
                        value = (Direction::from(i as u8), newval.1);
                    }
                    if newval.1 > alpha {
                        alpha = newval.1;
                    }
                    if alpha >= beta {
                        break;
                    }
                }
            }
        }
        value
    } else {
        let mut value = (Direction::Up, WIN);
        for d in Direction::iter() {
            let mut actions = actions;
            actions[ply] = d;
            let newval = async_alphabeta_rec(
                game,
                actions,
                depth,
                ply + 1,
                alpha,
                beta,
                heuristic.clone(),
            )
            .await;
            if newval.1 < value.1 {
                value = (d, newval.1);
            }
            if newval.1 < beta {
                beta = newval.1;
            }
            if alpha >= beta {
                break;
            }
        }
        value
    }
}

/// Alpha-Beta tree search.
///
/// @see https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
/// - Assumes the maximizing agent has id 0
/// - Assumes only two snakes are alive
pub fn alphabeta(game: &Game, depth: usize, heuristic: &dyn Heuristic) -> (Direction, f64) {
    alphabeta_rec(game, [Direction::Up; 4], depth, 0, LOSS, WIN, heuristic)
}

fn alphabeta_rec(
    game: &Game,
    actions: [Direction; 4],
    depth: usize,
    ply: usize,
    mut alpha: f64,
    mut beta: f64,
    heuristic: &dyn Heuristic,
) -> (Direction, f64) {
    if ply == game.snakes.len() {
        let mut game = game.clone();
        game.step(&actions);
        match game.outcome() {
            Outcome::Winner(0) => return (Direction::Up, WIN),
            Outcome::Winner(_) => return (Direction::Up, LOSS),
            Outcome::Match => return (Direction::Up, DRAW),
            Outcome::None => {}
        }

        if depth == 0 {
            (Direction::Up, heuristic.eval(&game))
        } else {
            alphabeta_rec(
                &game,
                [Direction::Up; 4],
                depth - 1,
                0,
                alpha,
                beta,
                heuristic,
            )
        }
    } else if ply == 0 {
        let mut value = (Direction::Up, LOSS);
        for d in Direction::iter() {
            let mut actions = actions;
            actions[ply] = d;
            let newval = alphabeta_rec(game, actions, depth, ply + 1, alpha, beta, heuristic);
            if newval.1 > value.1 {
                value = (d, newval.1);
            }
            if newval.1 > alpha {
                alpha = newval.1;
            }
            if alpha >= beta {
                break;
            }
        }
        value
    } else {
        let mut value = (Direction::Up, WIN);
        for d in Direction::iter() {
            let mut actions = actions;
            actions[ply] = d;
            let newval = alphabeta_rec(game, actions, depth, ply + 1, alpha, beta, heuristic);
            if newval.1 < value.1 {
                value = (d, newval.1);
            }
            if newval.1 < beta {
                beta = newval.1;
            }
            if alpha >= beta {
                break;
            }
        }
        value
    }
}
