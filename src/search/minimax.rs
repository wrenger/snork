use std::sync::Arc;

use crate::game::Game;
use crate::{env::Direction, game::Outcome};

use async_recursion::async_recursion;

use super::{Heuristic, DRAW, LOSS, WIN};

/// This algorithm is more or less a hacky variation of minmax with multiple agents.
/// The player with id 0 is the maximizing player, the others are minimizing.
///
/// The return value contains the heuristic for each of the four moves of the maximizing player.
///
/// If the maximizing player dies traversal ends and min is returned.
/// Dead enemies are skipped.
pub async fn async_max_n(game: &Game, depth: usize, heuristic: Arc<dyn Heuristic>) -> [f64; 4] {
    assert!(game.snakes.len() <= 4);
    async_max_n_rec(game, depth, 0, [Direction::Up; 4], heuristic).await
}

#[async_recursion]
async fn async_max_n_rec(
    game: &Game,
    depth: usize,
    ply: usize,
    actions: [Direction; 4],
    heuristic: Arc<dyn Heuristic>,
) -> [f64; 4] {
    if ply == game.snakes.len() {
        // simulate
        let mut game = game.clone();
        game.step(&actions[..]);

        match game.outcome() {
            Outcome::Winner(0) => return [WIN + heuristic.eval(&game), DRAW, DRAW, DRAW],
            Outcome::Winner(_) => return [LOSS; 4],
            Outcome::Match => return [DRAW; 4],
            Outcome::None => {}
        }

        if depth <= 1 {
            // eval
            [heuristic.eval(&game), DRAW, DRAW, DRAW]
        } else {
            let mut result =
                async_max_n_rec(&game, depth - 1, 0, [Direction::Up; 4], heuristic).await;
            // max
            for i in 1..4 {
                if result[i] > result[0] {
                    result[0] = result[i];
                }
            }
            result
        }
    } else if ply == 0 {
        // collect all outcomes instead of max
        let mut result = [LOSS; 4];

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
                async_max_n_rec(&game, depth, ply + 1, actions, heuristic).await
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
        let mut min = 2.0 * WIN;
        let mut moved = false;
        for d in Direction::iter() {
            if !game.move_is_valid(ply as u8, d) {
                continue;
            }

            let mut actions = actions;
            actions[ply] = d;
            let val = async_max_n_rec(game, depth, ply + 1, actions, heuristic.clone()).await[0];
            if val < min {
                min = val;
                moved = true;

                // skip if already lowest possible outcome
                if val <= LOSS {
                    break;
                }
            }
        }
        if !moved {
            // continue with next agent
            min = async_max_n_rec(game, depth, ply + 1, actions, heuristic).await[0];
        }
        [min, DRAW, DRAW, DRAW]
    }
}

/// This algorithm is more or less a hacky variation of minmax with multiple agents.
/// The player with id 0 is the maximizing player, the others are minimizing.
///
/// The return value contains the heuristic for each of the four moves of the maximizing player.
///
/// If the maximizing player dies traversal ends and min is returned.
/// Dead enemies are skipped.
pub fn max_n(game: &Game, depth: usize, heuristic: &dyn Heuristic) -> [f64; 4] {
    assert!(game.snakes.len() <= 4);
    max_n_rec(game, depth, 0, [Direction::Up; 4], heuristic)
}

fn max_n_rec(
    game: &Game,
    depth: usize,
    ply: usize,
    actions: [Direction; 4],
    heuristic: &dyn Heuristic,
) -> [f64; 4] {
    if ply == game.snakes.len() {
        // simulate
        let mut game = game.clone();
        game.step(&actions[..]);

        match game.outcome() {
            Outcome::Winner(0) => return [WIN + heuristic.eval(&game), DRAW, DRAW, DRAW],
            Outcome::Winner(_) => return [LOSS; 4],
            Outcome::Match => return [DRAW; 4],
            Outcome::None => {}
        }

        if depth <= 1 {
            // eval
            [heuristic.eval(&game), DRAW, DRAW, DRAW]
        } else {
            let mut result = max_n_rec(&game, depth - 1, 0, [Direction::Up; 4], heuristic);
            // max
            for i in 1..4 {
                if result[i] > result[0] {
                    result[0] = result[i];
                }
            }
            result
        }
    } else if ply == 0 {
        // collect all outcomes instead of max
        let mut result = [LOSS; 4];
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
        let mut min = 2.0 * WIN;
        let mut moved = false;
        for d in Direction::iter() {
            if !game.move_is_valid(ply as u8, d) {
                continue;
            }

            let mut actions = actions;
            actions[ply] = d;
            let val = max_n_rec(game, depth, ply + 1, actions, heuristic)[0];
            if val < min {
                min = val;
                moved = true;

                // skip if already lowest possible outcome
                if val <= LOSS {
                    break;
                }
            }
        }
        if !moved {
            // continue with next agent
            min = max_n_rec(game, depth, ply + 1, actions, heuristic)[0];
        }
        [min, DRAW, DRAW, DRAW]
    }
}

#[cfg(test)]
mod test {
    use log::info;

    use crate::floodfill::FloodFill;
    use crate::game::Game;
    use crate::logging;
    use crate::search::{alphabeta, Heuristic};

    #[derive(Debug, Clone, Default)]
    struct TestH;
    impl Heuristic for TestH {
        fn eval(&self, game: &Game) -> f64 {
            let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
            if game.snake_is_alive(0) {
                flood_fill.flood_snakes(&game.grid, &game.snakes);
                flood_fill.count_space(0) as f64
            } else {
                0.0
            }
        }
    }

    #[test]
    #[ignore]
    fn max_n() {
        use super::*;
        use crate::env::Vec2D;
        use crate::game::Snake;
        use std::time::Instant;
        logging();

        let snakes = vec![
            Snake::new(
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
                vec![Vec2D::new(3, 7), Vec2D::new(3, 6), Vec2D::new(3, 5)].into(),
                100,
            ),
            Snake::new(
                vec![Vec2D::new(10, 7), Vec2D::new(10, 6), Vec2D::new(10, 5)].into(),
                100,
            ),
            Snake::new(
                vec![Vec2D::new(10, 0), Vec2D::new(9, 0), Vec2D::new(8, 0)].into(),
                100,
            ),
        ];

        let game = Game::new(0, 11, 11, snakes, &[], &[]);
        info!("{:?}", game.grid);
        let start = Instant::now();

        let moves = max_n(&game, 3, &TestH);
        let end = Instant::now();
        info!("{:?}", moves);
        info!("time {}ms", (end - start).as_millis());
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn async_max_n() {
        use super::*;
        use crate::env::Vec2D;
        use crate::game::Snake;
        use std::time::Instant;
        logging();

        let snakes = vec![
            Snake::new(
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
                vec![Vec2D::new(3, 7), Vec2D::new(3, 6), Vec2D::new(3, 5)].into(),
                100,
            ),
            Snake::new(
                vec![Vec2D::new(10, 7), Vec2D::new(10, 6), Vec2D::new(10, 5)].into(),
                100,
            ),
            Snake::new(
                vec![Vec2D::new(10, 0), Vec2D::new(9, 0), Vec2D::new(8, 0)].into(),
                100,
            ),
        ];

        let game = Game::new(0, 11, 11, snakes, &[], &[]);
        info!("{:?}", game.grid);
        let start = Instant::now();
        let moves = async_max_n(&game, 3, Arc::new(TestH)).await;
        let end = Instant::now();
        info!("{:?}", moves);
        info!("async time {}ms", (end - start).as_millis());
    }

    #[test]
    #[ignore]
    fn duel() {
        use super::*;
        use crate::env::Vec2D;
        use crate::game::Snake;
        use std::time::Instant;
        logging();

        let snakes = vec![
            Snake::new(
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
                vec![Vec2D::new(3, 7), Vec2D::new(3, 6), Vec2D::new(3, 5)].into(),
                100,
            ),
        ];

        let game = Game::new(0, 11, 11, snakes, &[], &[]);
        info!("{:?}", game.grid);

        let start = Instant::now();
        let moves = max_n(&game, 6, &TestH);
        let end = Instant::now();
        info!("max_n {:?}", moves);
        info!("max_n time {}ms", (end - start).as_millis());

        let start = Instant::now();
        let moves = alphabeta(&game, 6, &TestH);
        let end = Instant::now();
        info!("alpha_beta {:?}", moves);
        info!("alpha_beta time {}ms", (end - start).as_millis());
    }
}
