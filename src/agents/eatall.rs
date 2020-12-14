use std::cmp::Reverse;

use rand::seq::IteratorRandom;

use super::util::*;
use super::Agent;
use crate::env::*;

#[derive(Debug, Default)]
pub struct EatAllAgent {
    state: usize,
}

impl Agent for EatAllAgent {
    fn start(&mut self, _: &GameRequest) {}

    fn step(&mut self, request: &GameRequest) -> MoveResponse {
        if let Some(you_i) = request.board.snakes.iter().position(|s| s == &request.you) {
            let snakes: Vec<CSnake> = request
                .board
                .snakes
                .iter()
                .enumerate()
                .map(|(i, s)| CSnake::new(i as _, s.body.clone()))
                .collect();
            let you_i = you_i as i8;
            let you: &CSnake = &snakes[you_i as usize];

            let mut grid = Grid::new(request.board.width, request.board.height);
            grid.add_snakes(&snakes);
            let space_after_move = grid.space_after_move(you_i, &snakes);

            // Find Food
            if request.turn < 50 || request.board.snakes[you_i as usize].health < 30 {
                // Add space to avoid enemy heads
                for (i, snake) in snakes.iter().enumerate() {
                    if i as i8 != you_i && you.body.len() <= snake.body.len() {
                        for dir in Direction::iter() {
                            let p = snake.head().apply(dir);
                            if grid.has(p) {
                                grid[p] = BOARD_OBSTACLE
                            }
                        }
                    }
                }

                // Heuristic for preferring high movement
                let first_move_costs = [
                    1.0 - space_after_move[0] as f64 / (grid.width * grid.height) as f64,
                    1.0 - space_after_move[1] as f64 / (grid.width * grid.height) as f64,
                    1.0 - space_after_move[2] as f64 / (grid.width * grid.height) as f64,
                    1.0 - space_after_move[3] as f64 / (grid.width * grid.height) as f64,
                ];

                grid.add_food(&request.board.food);
                use priority_queue::PriorityQueue;
                let mut food_dirs: PriorityQueue<Direction, Reverse<usize>> = PriorityQueue::new();
                for &p in &request.board.food {
                    if let Some(path) = grid.a_star(you.head(), p, first_move_costs) {
                        if path.len() >= 2 {
                            food_dirs.push(Direction::from(path[1] - path[0]), Reverse(path.len()));
                        }
                    }
                }

                while let Some((dir, _)) = food_dirs.pop() {
                    if space_after_move[dir as u8 as usize] > you.body.len() {
                        return MoveResponse::new(dir);
                    }
                }
            }

            if let Some((dir, space)) = space_after_move.iter().enumerate().max_by_key(|(_, v)| *v)
            {
                if *space > 0 {
                    let d: Direction = unsafe { std::mem::transmute(dir as u8) };
                    return MoveResponse::new(d);
                }
            }

            let mut rng = rand::thread_rng();
            MoveResponse::new(
                Direction::iter()
                    .filter(|&d| grid.avaliable(request.you.body[0].apply(d)))
                    .choose(&mut rng)
                    .unwrap_or(Direction::Up),
            )
        } else {
            MoveResponse::default()
        }
    }

    fn end(&mut self, _: &GameRequest) {}
}
