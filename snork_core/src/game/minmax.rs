use super::Game;
use crate::env::Direction;
use std::f64;

/// The result of a heuristic.
pub trait Comparable: Default + Copy + PartialOrd + std::fmt::Debug {
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
    // This vector is reused for the entire layer
    let mut action = Vec::with_capacity(game.snakes.len());
    max_n_rec(&game, depth, 0, &mut action, &mut heuristic)
}

fn max_n_rec<F, T>(
    game: &Game,
    depth: usize,
    ply: usize,
    actions: &mut Vec<Direction>,
    heuristic: &mut F,
) -> [T; 4]
where
    F: FnMut(&Game) -> T,
    T: Comparable,
{
    if ply == game.snakes.len() {
        assert_eq!(ply, actions.len());
        // simulate
        let mut game = game.clone();
        game.step(&actions);

        if depth <= 1 {
            // eval
            [heuristic(&game), T::default(), T::default(), T::default()]
        } else {
            let mut actions = Vec::with_capacity(game.snakes.len());
            let mut result = max_n_rec(&game, depth - 1, 0, &mut actions, heuristic);
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
        if game.snake_is_alive(ply as u8) {
            for (i, d) in Direction::iter().enumerate() {
                actions.truncate(ply);
                actions.push(d);
                result[i] = max_n_rec(game, depth, ply + 1, actions, heuristic)[0];
            }
        }
        result
    } else {
        let mut min = T::max();
        if game.snake_is_alive(ply as u8) {
            for d in Direction::iter() {
                actions.truncate(ply);
                actions.push(d);
                let val = max_n_rec(game, depth, ply + 1, actions, heuristic)[0];
                if val < min {
                    min = val;
                }
                // skip if already lowest possible outcome
                if val <= T::min() {
                    break;
                }
            }
        } else {
            // continue with next agent
            actions.truncate(ply);
            actions.push(Direction::Up);
            min = max_n_rec(game, depth, ply + 1, actions, heuristic)[0];
        }
        [min, T::default(), T::default(), T::default()]
    }
}

/// Alpha-Beta tree search.
/// @see https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
/// Assuming the maximizing agent has id 0
///
/// Generally faster for two agents but slower for more.
pub fn alphabeta<F, T>(game: &Game, depth: usize, mut heuristic: F) -> (Direction, T)
where
    F: FnMut(&Game) -> T,
    T: Comparable,
{
    let mut actions = Vec::new();
    alphabeta_rec(
        &game,
        &mut actions,
        depth,
        0,
        T::min(),
        T::max(),
        &mut heuristic,
    )
}

fn alphabeta_rec<F, T>(
    game: &Game,
    actions: &mut Vec<Direction>,
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
        game.step(actions);
        if depth == 0 {
            (Direction::Up, heuristic(&game))
        } else {
            let mut actions = Vec::new();
            alphabeta_rec(&game, &mut actions, depth - 1, 0, alpha, beta, heuristic)
        }
    } else if ply == 0 {
        let mut value = (Direction::Up, T::min());
        for d in Direction::iter() {
            actions.truncate(ply);
            actions.push(d);
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
            actions.truncate(ply);
            actions.push(d);
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

        let mut game = Game::new(11, 11);
        game.reset(snakes, &[], &[]);
        println!("{:?}", game.grid);
        let start = Instant::now();
        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
        let moves = max_n(&game, 2, |game| {
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

        let mut game = Game::new(11, 11);
        game.reset(snakes, &[], &[]);
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
