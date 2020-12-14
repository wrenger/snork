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
        if let Some(you) = request.board.snakes.iter().position(|s| s == &request.you) {
            let snakes = request
                .board
                .snakes
                .iter()
                .map(|s| CSnake::new(0, s.body.clone()))
                .collect::<Vec<_>>();

            let mut grid = Grid::filled(&request.board.food, &snakes);

            for (i, snake) in snakes.iter().enumerate() {
                if i != you && snakes[you].body.len() <= snake.body.len() {
                    for dir in Direction::iter() {
                        let p = snake.head().apply(dir);
                        if grid.has(p) {
                            grid[p] = BOARD_OBSTACLE
                        }
                    }
                }
            }

            use priority_queue::PriorityQueue;
            let mut food_dirs: PriorityQueue<Direction, Reverse<usize>> = PriorityQueue::new();
            for &p in &request.board.food {
                if let Some(path) = grid.a_star(request.you.head, p) {
                    if path.len() >= 2 {
                        food_dirs.push(Direction::from(path[1] - path[0]), Reverse(path.len()));
                    }
                }
            }
            if let Some((dir, _)) = food_dirs.pop() {
                return MoveResponse::new(dir);
            }

            let mut rng = rand::thread_rng();
            MoveResponse::new(
                Direction::iter()
                    .filter(|&d| grid.avaliable(request.you.head.apply(d)))
                    .choose(&mut rng)
                    .unwrap_or(Direction::Up),
            )
        } else {
            MoveResponse::default()
        }
    }

    fn end(&mut self, _: &GameRequest) {}
}

#[cfg(test)]
mod test {

    #[test]
    fn test_astar() {
        use super::*;
        let grid = Grid::new();
        let path = grid.a_star(Vec2D::new(0, 0), Vec2D::new(1, 1)).unwrap();
        println!("{:?}", path);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], Vec2D::new(0, 0));
        assert_eq!(path[2], Vec2D::new(1, 1));
    }
}
