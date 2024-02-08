use std::sync::Arc;

use crate::game::Game;
use crate::{env::Direction, game::Outcome};

use async_recursion::async_recursion;
use tokio::task::JoinSet;

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

    let mut set = JoinSet::new();
    for d in Direction::all() {
        if !game.move_is_valid(0, d) {
            continue;
        }

        let actions = [d, Direction::Up, Direction::Up, Direction::Up];
        let game = game.clone();
        let heuristic = heuristic.clone();

        // Create tasks for subtrees.
        set.spawn(async move {
            let r = async_max_n_rec(&game, depth, 1, actions, heuristic).await;
            (d, r)
        });
    }

    let mut result = [LOSS; 4];
    while let Some(r) = set.join_next().await {
        if let Ok((d, r)) = r {
            result[d as usize] = r;
        }
    }
    result
}

#[async_recursion]
async fn async_max_n_rec(
    game: &Game,
    depth: usize,
    ply: usize,
    actions: [Direction; 4],
    heuristic: Arc<dyn Heuristic>,
) -> f64 {
    if ply == game.snakes.len() {
        // simulate
        let mut game = game.clone();
        game.step(&actions[..]);

        match game.outcome() {
            Outcome::Winner(0) => return WIN + heuristic.eval(&game),
            Outcome::Winner(_) => return LOSS,
            Outcome::Match => return DRAW,
            Outcome::None => {}
        }

        if depth <= 1 {
            // eval
            heuristic.eval(&game)
        } else {
            async_max_n_rec(&game, depth - 1, 0, [Direction::Up; 4], heuristic).await
        }
    } else if ply == 0 {
        // max
        let mut set = JoinSet::new();
        for d in Direction::all() {
            if !game.move_is_valid(0, d) {
                continue;
            }

            let actions = [d, Direction::Up, Direction::Up, Direction::Up];
            let game = game.clone();
            let heuristic = heuristic.clone();

            // Create tasks for subtrees.
            set.spawn(
                async move { async_max_n_rec(&game, depth, ply + 1, actions, heuristic).await },
            );
        }

        let mut max = LOSS;
        while let Some(r) = set.join_next().await {
            if let Ok(r) = r {
                max = max.max(r);
            }
        }
        max
    } else {
        // min
        let mut min = 2.0 * WIN;
        let mut moved = false;
        for d in Direction::all() {
            if !game.move_is_valid(ply as u8, d) {
                continue;
            }

            let mut actions = actions;
            actions[ply] = d;
            let val = async_max_n_rec(game, depth, ply + 1, actions, heuristic.clone()).await;
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
            min = async_max_n_rec(game, depth, ply + 1, actions, heuristic).await;
        }
        min
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
    let mut result = [LOSS; 4];
    for d in Direction::all() {
        if game.move_is_valid(0, d) {
            let actions = [d, Direction::Up, Direction::Up, Direction::Up];
            result[d as usize] = max_n_rec(game, depth, 1, actions, heuristic);
        }
    }
    result
}

fn max_n_rec(
    game: &Game,
    depth: usize,
    ply: usize,
    actions: [Direction; 4],
    heuristic: &dyn Heuristic,
) -> f64 {
    if ply == game.snakes.len() {
        // simulate
        let mut game = game.clone();
        game.step(&actions[..]);

        match game.outcome() {
            Outcome::Winner(0) => return WIN + heuristic.eval(&game),
            Outcome::Winner(_) => return LOSS,
            Outcome::Match => return DRAW,
            Outcome::None => {}
        }

        if depth <= 1 {
            // eval
            heuristic.eval(&game)
        } else {
            max_n_rec(&game, depth - 1, 0, [Direction::Up; 4], heuristic)
        }
    } else if ply == 0 {
        // collect all outcomes instead of max
        let mut max = LOSS;
        for d in Direction::all() {
            if !game.move_is_valid(0, d) {
                continue;
            }
            let mut actions = actions;
            actions[ply] = d;
            max = max.max(max_n_rec(game, depth, ply + 1, actions, heuristic));
        }
        max
    } else {
        let mut min = 2.0 * WIN;
        let mut moved = false;
        for d in Direction::all() {
            if !game.move_is_valid(ply as u8, d) {
                continue;
            }

            let mut actions = actions;
            actions[ply] = d;
            let val = max_n_rec(game, depth, ply + 1, actions, heuristic);
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
            min = max_n_rec(game, depth, ply + 1, actions, heuristic);
        }
        min
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
