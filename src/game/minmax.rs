use super::Game;
use crate::{env::Direction, game::Outcome};

use async_recursion::async_recursion;

/// The result of a heuristic.
pub trait Comparable: Default + Copy + PartialOrd + std::fmt::Debug + Send {
    fn max() -> Self;
    fn min() -> Self;
}

impl Comparable for f64 {
    fn max() -> f64 {
        std::f64::INFINITY
    }
    fn min() -> f64 {
        -std::f64::INFINITY
    }
}

pub async fn async_max_n<F, T>(game: &Game, depth: usize, heuristic: F) -> [T; 4]
where
    F: Fn(&Game) -> T + Send + Sync + Clone + 'static,
    T: Comparable + 'static,
{
    assert!(game.snakes.len() <= 4);
    async_max_n_rec(&game, depth, 0, [Direction::Up; 4], &heuristic).await
}

#[async_recursion]
async fn async_max_n_rec<F, T>(
    game: &Game,
    depth: usize,
    ply: usize,
    actions: [Direction; 4],
    heuristic: &F,
) -> [T; 4]
where
    F: Fn(&Game) -> T + Send + Sync + Clone + 'static,
    T: Comparable + 'static,
{
    if ply == game.snakes.len() {
        // simulate
        let mut game = game.clone();
        game.step(&actions[..]);

        match game.outcome() {
            Outcome::Winner(0) => return [T::max(); 4],
            Outcome::Winner(_) => return [T::min(); 4],
            Outcome::Match => return [T::min(); 4],
            Outcome::None => {}
        }

        if depth <= 1 {
            // eval
            [heuristic(&game), T::default(), T::default(), T::default()]
        } else {
            let mut result =
                async_max_n_rec(&game, depth - 1, 0, [Direction::Up; 4], heuristic).await;
            // max
            for i in 1..4 {
                if result[i] > result[0] {
                    result[0] = result[i]
                }
            }
            result
        }
    } else if ply == 0 {
        // collect all outcomes instead of max
        let mut result = [T::min(); 4];

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
                async_max_n_rec(&game, depth, ply + 1, actions, &heuristic).await
            }));
        }
        for (i, future) in futures.into_iter().enumerate() {
            if let Some(f) = future {
                if let Ok(r) = f.await {
                    result[i] = r[0];
                }
            }
        }

        result
    } else {
        let mut min = T::max();
        let mut moved = false;
        for d in Direction::iter() {
            if !game.move_is_valid(ply as u8, d) {
                continue;
            }
            moved = true;

            let mut actions = actions.clone();
            actions[ply] = d;
            let val = async_max_n_rec(game, depth, ply + 1, actions, heuristic).await[0];
            if val < min {
                min = val;
            }
            // skip if already lowest possible outcome
            if val <= T::min() {
                break;
            }
        }
        if !moved {
            // continue with next agent
            min = async_max_n_rec(game, depth, ply + 1, actions, heuristic).await[0];
        }
        [min, T::default(), T::default(), T::default()]
    }
}

/// This algorithm is more or less a hacky variation of minmax with multiple agents.
/// The player with id 0 is the maximizing player, the others are minimizing.
///
/// The return value contains the heuristic for each of the four moves of the maximizing player.
///
/// If the maximizing player dies traversal ends and min is returned.
/// Dead enemies are skipped.
pub fn max_n<F, T>(game: &Game, depth: usize, mut heuristic: F) -> [T; 4]
where
    F: FnMut(&Game) -> T,
    T: Comparable,
{
    assert!(game.snakes.len() <= 4);
    max_n_rec(&game, depth, 0, [Direction::Up; 4], &mut heuristic)
}

fn max_n_rec<F, T>(
    game: &Game,
    depth: usize,
    ply: usize,
    actions: [Direction; 4],
    heuristic: &mut F,
) -> [T; 4]
where
    F: FnMut(&Game) -> T,
    T: Comparable,
{
    if ply == game.snakes.len() {
        // simulate
        let mut game = game.clone();
        game.step(&actions[..]);

        match game.outcome() {
            Outcome::Winner(0) => return [T::max(); 4],
            Outcome::Winner(_) => return [T::min(); 4],
            Outcome::Match => return [T::min(); 4],
            Outcome::None => {}
        }

        if depth <= 1 {
            // eval
            [heuristic(&game), T::default(), T::default(), T::default()]
        } else {
            let mut result = max_n_rec(&game, depth - 1, 0, [Direction::Up; 4], heuristic);
            // max
            for i in 1..4 {
                if result[i] > result[0] {
                    result[0] = result[i]
                }
            }
            result
        }
    } else if ply == 0 {
        // collect all outcomes instead of max
        let mut result = [T::min(); 4];
        for d in Direction::iter() {
            if !game.move_is_valid(0, d) {
                continue;
            }
            let mut actions = actions;
            actions[ply] = d;
            result[d as u8 as usize] = max_n_rec(game, depth, ply + 1, actions, heuristic)[0];
        }
        result
    } else {
        let mut min = T::max();
        let mut moved = false;
        for d in Direction::iter() {
            if !game.move_is_valid(ply as u8, d) {
                continue;
            }
            moved = true;

            let mut actions = actions;
            actions[ply] = d;
            let val = max_n_rec(game, depth, ply + 1, actions, heuristic)[0];
            if val < min {
                min = val;
            }
            // skip if already lowest possible outcome
            if val <= T::min() {
                break;
            }
        }
        if !moved {
            // continue with next agent
            min = max_n_rec(game, depth, ply + 1, actions, heuristic)[0];
        }
        [min, T::default(), T::default(), T::default()]
    }
}

pub async fn async_alphabeta<F, T>(game: &Game, depth: usize, mut heuristic: F) -> (Direction, T)
where
    F: Fn(&Game) -> T + Send + Sync + Clone + 'static,
    T: Comparable + 'static,
{
    async_alphabeta_rec(
        &game,
        [Direction::Up; 4],
        depth,
        0,
        T::min(),
        T::max(),
        &mut heuristic,
    )
    .await
}

#[async_recursion]
async fn async_alphabeta_rec<F, T>(
    game: &Game,
    actions: [Direction; 4],
    depth: usize,
    ply: usize,
    mut alpha: T,
    mut beta: T,
    heuristic: &F,
) -> (Direction, T)
where
    F: Fn(&Game) -> T + Send + Sync + Clone + 'static,
    T: Comparable + 'static,
{
    if ply == game.snakes.len() {
        let mut game = game.clone();
        game.step(&actions);
        if depth == 0 {
            (Direction::Up, heuristic(&game))
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
        let mut value = (Direction::Up, T::min());

        let mut futures = [None, None, None, None];
        for d in Direction::iter() {
            let game = game.clone();
            let heuristic = heuristic.clone();
            let mut actions = actions;
            actions[ply] = d;
            futures[d as u8 as usize] = Some(tokio::task::spawn(async move {
                async_alphabeta_rec(&game, actions, depth, ply + 1, alpha, beta, &heuristic).await
            }));
        }

        for (i, future) in futures.into_iter().enumerate() {
            let newval = future.unwrap().await.unwrap();

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
        value
    } else {
        let mut value = (Direction::Up, T::max());
        for d in Direction::iter() {
            let mut actions = actions.clone();
            actions[ply] = d;
            let newval =
                async_alphabeta_rec(game, actions, depth, ply + 1, alpha, beta, heuristic).await;
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
/// @see https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
/// Assuming the maximizing agent has id 0
/// Assuming only two snakes are alive
pub fn alphabeta<F, T>(game: &Game, depth: usize, mut heuristic: F) -> (Direction, T)
where
    F: FnMut(&Game) -> T,
    T: Comparable,
{
    alphabeta_rec(
        &game,
        [Direction::Up; 4],
        depth,
        0,
        T::min(),
        T::max(),
        &mut heuristic,
    )
}

fn alphabeta_rec<F, T>(
    game: &Game,
    actions: [Direction; 4],
    depth: usize,
    ply: usize,
    mut alpha: T,
    mut beta: T,
    heuristic: &mut F,
) -> (Direction, T)
where
    F: FnMut(&Game) -> T,
    T: Comparable,
{
    if ply == game.snakes.len() {
        let mut game = game.clone();
        game.step(&actions);
        if depth == 0 {
            (Direction::Up, heuristic(&game))
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
        let mut value = (Direction::Up, T::min());
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
        let mut value = (Direction::Up, T::max());
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

#[cfg(test)]
mod test {

    #[test]
    #[ignore]
    fn max_n() {
        use super::super::{FloodFill, Snake};
        use super::*;
        use crate::env::Vec2D;
        use std::time::Instant;

        let snakes = vec![
            Snake::new(
                0,
                vec![
                    Vec2D::new(0, 3),
                    Vec2D::new(1, 3),
                    Vec2D::new(2, 3),
                    Vec2D::new(3, 3),
                ]
                .into(),
                100,
            ),
            Snake::new(
                1,
                vec![Vec2D::new(3, 7), Vec2D::new(3, 6), Vec2D::new(3, 5)].into(),
                100,
            ),
            Snake::new(
                2,
                vec![Vec2D::new(10, 7), Vec2D::new(10, 6), Vec2D::new(10, 5)].into(),
                100,
            ),
            Snake::new(
                3,
                vec![Vec2D::new(10, 0), Vec2D::new(9, 0), Vec2D::new(8, 0)].into(),
                100,
            ),
        ];

        let game = Game::new(11, 11, snakes, &[], &[]);
        println!("{:?}", game.grid);
        let start = Instant::now();
        let moves = max_n(&game, 3, |game| {
            let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
            if game.snake_is_alive(0) {
                flood_fill.flood_snakes(&game.grid, &game.snakes);
                flood_fill.count_space(true) as f64
            } else {
                0.0
            }
        });
        let end = Instant::now();
        println!("{:?}", moves);
        println!("time {}ms", (end - start).as_millis());
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn async_max_n() {
        use super::super::{FloodFill, Snake};
        use super::*;
        use crate::env::Vec2D;
        use std::time::Instant;

        let snakes = vec![
            Snake::new(
                0,
                vec![
                    Vec2D::new(0, 3),
                    Vec2D::new(1, 3),
                    Vec2D::new(2, 3),
                    Vec2D::new(3, 3),
                ]
                .into(),
                100,
            ),
            Snake::new(
                1,
                vec![Vec2D::new(3, 7), Vec2D::new(3, 6), Vec2D::new(3, 5)].into(),
                100,
            ),
            Snake::new(
                2,
                vec![Vec2D::new(10, 7), Vec2D::new(10, 6), Vec2D::new(10, 5)].into(),
                100,
            ),
            Snake::new(
                3,
                vec![Vec2D::new(10, 0), Vec2D::new(9, 0), Vec2D::new(8, 0)].into(),
                100,
            ),
        ];

        let game = Game::new(11, 11, snakes, &[], &[]);
        println!("{:?}", game.grid);
        let start = Instant::now();
        let moves = async_max_n(&game, 3, |game| {
            let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
            if game.snake_is_alive(0) {
                flood_fill.flood_snakes(&game.grid, &game.snakes);
                flood_fill.count_space(true) as f64
            } else {
                0.0
            }
        })
        .await;
        let end = Instant::now();
        println!("{:?}", moves);
        println!("async time {}ms", (end - start).as_millis());
    }

    #[test]
    #[ignore]
    fn duel() {
        use super::super::{FloodFill, Snake};
        use super::*;
        use crate::env::Vec2D;
        use std::time::Instant;

        let snakes = vec![
            Snake::new(
                0,
                vec![
                    Vec2D::new(0, 3),
                    Vec2D::new(1, 3),
                    Vec2D::new(2, 3),
                    Vec2D::new(3, 3),
                ]
                .into(),
                100,
            ),
            Snake::new(
                1,
                vec![Vec2D::new(3, 7), Vec2D::new(3, 6), Vec2D::new(3, 5)].into(),
                100,
            ),
        ];

        let game = Game::new(11, 11, snakes, &[], &[]);
        println!("{:?}", game.grid);

        let start = Instant::now();
        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
        let moves = max_n(&game, 6, |game| {
            if game.snake_is_alive(0) {
                flood_fill.flood_snakes(&game.grid, &game.snakes);
                flood_fill.count_space(true) as f64
            } else {
                -f64::INFINITY
            }
        });
        let end = Instant::now();
        println!("max_n {:?}", moves);
        println!("max_n time {}ms", (end - start).as_millis());

        let start = Instant::now();
        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
        let moves = alphabeta(&game, 6, |game| {
            if game.snake_is_alive(0) {
                flood_fill.flood_snakes(&game.grid, &game.snakes);
                flood_fill.count_space(true) as f64
            } else {
                -f64::INFINITY
            }
        });
        let end = Instant::now();
        println!("alpha_beta {:?}", moves);
        println!("alpha_beta time {}ms", (end - start).as_millis());
    }
}
