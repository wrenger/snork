use crate::env::*;
use crate::game::{Game, Outcome};

use super::{Heuristic, DRAW, LOSS, WIN};

use async_recursion::async_recursion;

pub async fn expectimax<H: Heuristic>(
    game: &Game,
    depth: usize,
    heuristic: &H,
) -> (Direction, f64) {
    assert!(game.snakes.len() <= 4);
    let (dir, h, _count) = expectimax_rec(game, depth, 0, [Direction::Up; 4], heuristic).await;
    (dir, h)
}

#[async_recursion]
async fn expectimax_rec<H: Heuristic>(
    game: &Game,
    depth: usize,
    ply: usize,
    actions: [Direction; 4],
    heuristic: &H,
) -> (Direction, f64, usize) {
    if ply == game.snakes.len() {
        // simulate
        let mut game = game.clone();
        game.step(&actions[..]);

        match game.outcome() {
            Outcome::Winner(0) => return (Direction::Up, WIN, 1),
            Outcome::Winner(_) => return (Direction::Up, LOSS, 1),
            Outcome::Match => return (Direction::Up, DRAW, 1),
            Outcome::None => {}
        }

        if depth <= 1 {
            // eval
            (Direction::Up, heuristic.eval(&game), 1)
        } else {
            expectimax_rec(&game, depth - 1, 0, [Direction::Up; 4], heuristic).await
        }
    } else if ply == 0 {
        // create tasks for all actions
        let mut futures = [None, None, None, None];

        for d in Direction::iter() {
            if !game.move_is_valid(0, d) {
                continue;
            }

            let actions = [d, Direction::Up, Direction::Up, Direction::Up];
            let game = game.clone();
            let heuristic = heuristic.clone();

            // Create tasks for subtrees.
            futures[d as u8 as usize] = Some(tokio::task::spawn(async move {
                expectimax_rec(&game, depth, ply + 1, actions, &heuristic).await
            }));
        }
        let mut total = 0.0;
        let mut max = LOSS;
        let mut dir = Direction::Up;
        for (i, future) in futures.into_iter().enumerate() {
            if let Some(f) = future {
                if let Ok((_, val, num)) = f.await {
                    if max < val {
                        max = val;
                        dir = Direction::from(i as u8)
                    }
                    total += val / num as f64;
                } else {
                    total += LOSS
                }
            } else {
                total += LOSS
            }
        }

        (dir, total / 4.0, 0)
    } else {
        let mut total = 0.0;
        let mut count = 0;
        let mut moved = false;
        for d in Direction::iter() {
            if !game.move_is_valid(ply as u8, d) {
                total += LOSS;
                continue;
            }
            let mut actions = actions;
            actions[ply] = d;
            let (_, val, c) = expectimax_rec(game, depth, ply + 1, actions, heuristic).await;
            total += val;
            count += c;
            moved = true;
        }
        if !moved {
            // continue with next agent
            return expectimax_rec(game, depth, ply + 1, actions, heuristic).await;
        }
        (Direction::Up, total, count)
    }
}
