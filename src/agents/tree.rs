use std::time::Instant;

use super::Agent;
use crate::env::*;
use crate::game::{max_n, Max, Cell, FloodFill, Game, Outcome, Snake};

use crate::util::argmax;

use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};

#[derive(Debug, Default)]
pub struct TreeAgent;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct HeuristicResult(f64, f64, f64, f64);

impl From<HeuristicResult> for f64 {
    fn from(v: HeuristicResult) -> f64 {
        v.0 + v.1 + v.2 + v.3
    }
}
impl PartialOrd for HeuristicResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        f64::from(*self).partial_cmp(&f64::from(*other))
    }
}
impl Max for HeuristicResult {
    fn max() -> HeuristicResult {
        HeuristicResult(std::f64::INFINITY, 0.0, 0.0, 0.0)
    }
}

impl TreeAgent {
    fn heuristic(
        &self,
        food: &[Vec2D],
        flood_fill: &mut FloodFill,
        game: &Game,
    ) -> HeuristicResult {
        match game.outcome() {
            Outcome::Match => HeuristicResult::default(),
            Outcome::Winner(0) => HeuristicResult(std::f64::INFINITY, 0.0, 0.0, 0.0),
            Outcome::Winner(_) => HeuristicResult(-std::f64::INFINITY, 0.0, 0.0, 0.0),
            Outcome::None => {
                flood_fill.flood_snakes(&game.grid, &game.snakes, 0);
                let space = flood_fill.count_space_of(true);

                let mobility = space as f64 / (game.grid.width * game.grid.height) as f64;

                let health = game.snakes[0].health as f64 / 100.0;

                let own_len = game.snakes[0].body.len();
                let max_enemy_len = game.snakes[1..]
                    .iter()
                    .map(|s| s.body.len())
                    .max()
                    .unwrap_or(0);
                let len_advantage = own_len as f64 / max_enemy_len as f64;

                let accessable_food =
                    food.iter().filter(|&&p| flood_fill[p].is_you()).count() as f64;

                HeuristicResult(mobility, health, len_advantage, accessable_food)
            }
        }
    }
}

impl Agent for TreeAgent {
    fn start(&mut self, _: &GameRequest) {}

    fn step(&mut self, request: &GameRequest) -> MoveResponse {
        let mut snakes = Vec::with_capacity(4);
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
        let mut game = Game::new(request.board.width, request.board.height);
        game.reset(snakes, &request.board.food);

        let depth = match game.snakes.len() {
            1 => 6,
            2 => 4,
            3 => 3,
            _ => 2,
        };

        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);

        let start = Instant::now();
        let evaluation = max_n(&game, depth, |game| {
            self.heuristic(&request.board.food, &mut flood_fill, game)
        });
        println!(
            "max_n {} {:?}ms {:?}",
            depth,
            (Instant::now() - start).as_millis(),
            evaluation
        );

        if let Some(dir) = argmax(evaluation.iter()) {
            if evaluation[dir] > HeuristicResult::default() {
                return MoveResponse::new(Direction::from(dir as u8));
            }
        }

        let mut rng = SmallRng::from_entropy();
        MoveResponse::new(
            game.valid_moves(0)
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
    fn bench_tree() {
        use super::*;
        use std::time::Instant;

        let game_req: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
        ).unwrap();

        let mut agent = TreeAgent::default();
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
