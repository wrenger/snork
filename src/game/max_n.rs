use super::Game;
use crate::env::Direction;

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

/// Assuming the evaluated agent has id = 0
pub fn max_n<F, T>(game: &Game, depth: usize, mut heuristic: F) -> [T; 4]
where
    F: FnMut(&Game) -> T,
    T: Comparable,
{
    let mut action = Vec::with_capacity(game.snakes.len());
    max_n_rec(&game, depth, 0, 0, &mut action, &mut heuristic)
}

fn max_n_rec<F, T>(
    game: &Game,
    depth: usize,
    current_depth: usize,
    current_ply_depth: usize,
    actions: &mut Vec<Direction>,
    heuristic: &mut F,
) -> [T; 4]
where
    F: FnMut(&Game) -> T,
    T: Comparable,
{
    if current_ply_depth == game.snakes.len() {

        assert_eq!(current_ply_depth, actions.len());
        // simulate
        let mut game = game.clone();
        game.step(&actions);

        if current_depth + 1 >= depth {
            // eval
            [heuristic(&game), T::default(), T::default(), T::default()]
        } else {
            let mut actions = Vec::with_capacity(game.snakes.len());
            let mut result = max_n_rec(&game, depth, current_depth + 1, 0, &mut actions, heuristic);
            // max
            for i in 1..4 {
                if result[i] > result[0] {
                    result[0] = result[i]
                }
            }
            result
        }
    } else if current_ply_depth == 0 {
        let mut result = [T::min(); 4];
        if game.snake_is_alive(current_ply_depth as u8) {
            for (i, d) in Direction::iter().enumerate() {
                actions.truncate(current_ply_depth);
                actions.push(d);
                result[i] = max_n_rec(
                    game,
                    depth,
                    current_depth,
                    current_ply_depth + 1,
                    actions,
                    heuristic,
                )[0];
            }
        }
        result
    } else {
        let mut min = T::max();
        if game.snake_is_alive(current_ply_depth as u8) {
            for d in Direction::iter() {
                actions.truncate(current_ply_depth);
                actions.push(d);
                let val = max_n_rec(
                    game,
                    depth,
                    current_depth,
                    current_ply_depth + 1,
                    actions,
                    heuristic,
                )[0];
                if val < min {
                    min = val;
                }
            }
        } else {
            actions.truncate(current_ply_depth);
            actions.push(Direction::Up);
            min = max_n_rec(
                game,
                depth,
                current_depth,
                current_ply_depth + 1,
                actions,
                heuristic,
            )[0];
        }
        [min, T::default(), T::default(), T::default()]
    }
}

#[cfg(test)]
mod test {

    #[test]
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
        game.reset(snakes, &[]);
        println!("{:?}", game.grid);
        let start = Instant::now();
        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
        let moves = max_n(&game, 2, |game| {
            if game.snake_is_alive(0) {
                flood_fill.flood_snakes(&game.grid, &game.snakes, 0);
                flood_fill.count_space_of(true) as f64
            } else {
                0.0
            }
        });
        let end = Instant::now();
        println!("{:?}", moves);
        println!("time {}ms", (end - start).as_millis());
    }
}
