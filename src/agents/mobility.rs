use std::cmp::Reverse;
use std::time::Instant;

use rand::seq::IteratorRandom;
use rand::{rngs::SmallRng, SeedableRng};

use super::Agent;
use crate::env::*;
use crate::game::{max_n, Cell, FloodFill, Game, Grid, Snake};

use crate::util::argmax;

#[derive(Debug)]
pub struct MobilityAgent {
    game: Game,
    flood_fill: FloodFill,
}

impl MobilityAgent {
    pub fn new(request: &GameRequest) -> MobilityAgent {
        MobilityAgent {
            game: Game::new(request.board.width, request.board.width),
            flood_fill: FloodFill::new(request.board.width, request.board.width),
        }
    }

    fn find_food(
        &self,
        food: &[Vec2D],
        grid: &Grid,
        space_after_move: &[f64; 4],
        first_move_costs: &[f64; 4],
    ) -> Option<Direction> {
        let you: &Snake = &self.game.snakes[0];

        use priority_queue::PriorityQueue;
        let mut food_dirs: PriorityQueue<Direction, Reverse<usize>> = PriorityQueue::new();
        for &p in food {
            if let Some(path) = grid.a_star(you.head(), p, first_move_costs) {
                if path.len() >= 2 {
                    let costs = path.len() + if self.flood_fill[p].is_you() { 0 } else { 5 };
                    food_dirs.push(Direction::from(path[1] - path[0]), Reverse(costs));
                }
            }
        }

        while let Some((dir, _)) = food_dirs.pop() {
            if space_after_move[dir as u8 as usize] as usize >= you.body.len() - 1 {
                return Some(dir);
            }
        }
        None
    }
}

impl Agent for MobilityAgent {
    fn start(&mut self, _: &GameRequest) {}

    fn step(&mut self, request: &GameRequest) -> MoveResponse {
        // Prepare grid
        let mut snakes = Vec::with_capacity(0);
        snakes.push(Snake::from(&request.you, 0));
        snakes.extend(
            request
                .board
                .snakes
                .iter()
                .filter(|s| s.id != request.you.id)
                .enumerate()
                .map(|(i, s)| Snake::from(s, i as u8 + 1)),
        );
        self.game.reset(snakes, &request.board.food);
        let you = &self.game.snakes[0];

        // Flood fill heuristics
        let start = Instant::now();
        let flood_fill = &mut self.flood_fill;
        let space_after_move = max_n(&self.game, 1, |game| {
            if game.snake_is_alive(0) {
                flood_fill.flood_snakes(&game.grid, &game.snakes, 0);
                flood_fill.count_space_of(true) as f64
            } else {
                0.0
            }
        });
        println!("max_n {:?}ms", (Instant::now() - start).as_millis());
        self.flood_fill
            .flood_snakes(&self.game.grid, &self.game.snakes, 0);

        // Avoid longer enemy heads
        let mut grid = self.game.grid.clone();
        for (i, snake) in self.game.snakes.iter().enumerate() {
            if 0 != i as u8 && snake.body.len() >= you.body.len() {
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

        // Find Food
        if you.body.len() < 10 || you.health < 35 {
            if let Some(dir) = self.find_food(
                &request.board.food,
                &grid,
                &space_after_move,
                &first_move_costs,
            ) {
                return MoveResponse::new(dir);
            }
        }

        if let Some(dir) = argmax(space_after_move.iter()) {
            if space_after_move[dir] > 0.0 {
                let d: Direction = unsafe { std::mem::transmute(dir as u8) };
                return MoveResponse::new(d);
            }
        }

        let mut rng = SmallRng::from_entropy();
        MoveResponse::new(
            self.game
                .valid_moves(0)
                .choose(&mut rng)
                .unwrap_or(Direction::Up),
        )
    }

    fn end(&mut self, _: &GameRequest) {}
}

#[cfg(test)]
mod test {
    #[test]
    #[ignore]
    fn bench_mobility() {
        use super::*;
        use std::time::Instant;

        let game_req: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
        ).unwrap();

        let mut agent = MobilityAgent::new(&game_req);
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
            runtime as f64 / COUNT as f64
        )
    }
}
