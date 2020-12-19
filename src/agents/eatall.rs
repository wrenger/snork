use std::cmp::Reverse;

use rand::seq::IteratorRandom;
use rand::{rngs::SmallRng, SeedableRng};

use super::Agent;
use crate::env::*;
use crate::game::{Cell, FloodFill, Grid};

#[derive(Debug)]
pub struct EatAllAgent {
    grid: Grid,
    floodfill: FloodFill,
    rng: rand::rngs::SmallRng,
}

impl EatAllAgent {
    pub fn new(request: &GameRequest) -> EatAllAgent {
        EatAllAgent {
            grid: Grid::new(request.board.width, request.board.width),
            floodfill: FloodFill::new(request.board.width, request.board.width),
            rng: SmallRng::from_entropy(),
        }
    }

    fn find_food(
        &self,
        food: &[Vec2D],
        snakes: &[SnakeData],
        you_i: u8,
        space_after_move: &[usize; 4],
    ) -> Option<Direction> {
        let you: &SnakeData = &snakes[you_i as usize];

        // Avoid longer enemy heads
        let mut grid = self.grid.clone();
        for (i, snake) in snakes.iter().enumerate() {
            if you_i != i as u8 && snake.body.len() >= you.body.len() {
                for d in Direction::iter() {
                    let p = snake.head().apply(d);
                    if grid.has(p) {
                        grid[p] = Cell::Occupied;
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

        use priority_queue::PriorityQueue;
        let mut food_dirs: PriorityQueue<Direction, Reverse<usize>> = PriorityQueue::new();
        for &p in food {
            if let Some(path) = grid.a_star(you.head(), p, first_move_costs) {
                if path.len() >= 2 {
                    let costs = path.len()
                        + if self.floodfill[p] == you_i as i8 {
                            0
                        } else {
                            5
                        };
                    food_dirs.push(Direction::from(path[1] - path[0]), Reverse(costs));
                }
            }
        }

        while let Some((dir, _)) = food_dirs.pop() {
            if space_after_move[dir as u8 as usize] >= you.body.len() - 1 {
                return Some(dir);
            }
        }
        None
    }
}

impl Agent for EatAllAgent {
    fn start(&mut self, _: &GameRequest) {}

    fn step(&mut self, request: &GameRequest) -> MoveResponse {
        if let Some(you_i) = request.board.snakes.iter().position(|s| s == &request.you) {
            let snakes = &request.board.snakes;
            let you_i = you_i as u8;
            let you = &snakes[you_i as usize];

            // Prepare grid
            self.grid.clear();
            for snake in snakes {
                self.grid.add_snake(snake.body.iter().cloned());
            }
            self.grid.add_food(&request.board.food);

            // Flood fill heuristics
            let space_after_move = self.floodfill.space_after_move(&self.grid, you_i, &snakes);
            self.floodfill.flood_snakes(&self.grid, &snakes, you_i);

            // Find Food
            if you.body.len() < 10 || request.board.snakes[you_i as usize].health < 35 {
                if let Some(dir) =
                    self.find_food(&request.board.food, &snakes, you_i, &space_after_move)
                {
                    return MoveResponse::new(dir);
                }
            }

            if let Some((dir, space)) = space_after_move.iter().enumerate().max_by_key(|(_, v)| *v)
            {
                if *space > 0 {
                    let d: Direction = unsafe { std::mem::transmute(dir as u8) };
                    return MoveResponse::new(d);
                }
            }

            let grid = &self.grid;
            let rng = &mut self.rng;
            MoveResponse::new(
                Direction::iter()
                    .filter(|&d| {
                        grid.has(you.head().apply(d)) && grid[you.head().apply(d)] != Cell::Occupied
                    })
                    .choose(rng)
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
    fn bench_eatall() {
        use super::*;
        use std::time::Instant;

        let game_req: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
        ).unwrap();

        let mut agent = EatAllAgent::new(&game_req);
        agent.start(&game_req);

        const COUNT: usize = 1000;

        let start = Instant::now();
        for _ in 0..COUNT {
            let d = agent.step(&game_req);
            assert_eq!(d.r#move, Direction::Down);
        }
        let end = Instant::now();
        let runtime = (end - start).as_millis();
        println!(
            "Runtime: total={}ms, avg={}ms",
            runtime,
            runtime / COUNT as u128
        )
    }
}
