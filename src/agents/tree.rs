use std::fmt;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use super::Agent;
use crate::env::*;
use crate::game::{max_n, Comparable, FloodFill, Game, Outcome, Snake};

use crate::util::argmax;

use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};

#[derive(Debug, Default)]
pub struct TreeAgent {
    config: TreeConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TreeConfig {
    mobility: f64,
    mobility_decay: f64,
    health: f64,
    health_decay: f64,
    len_advantage: f64,
    len_advantage_decay: f64,
    food_ownership: f64,
    food_ownership_decay: f64,
    centrality: f64,
    centrality_decay: f64,
}

impl Default for TreeConfig {
    fn default() -> Self {
        TreeConfig {
            mobility: 1.0,
            mobility_decay: 0.0,
            health: 1.0,
            health_decay: 0.0,
            len_advantage: 1.0,
            len_advantage_decay: 0.0,
            food_ownership: 1.0,
            food_ownership_decay: 0.0,
            centrality: 1.0,
            centrality_decay: 0.0,
        }
    }
}

#[derive(PartialEq, Default, Clone, Copy)]
pub struct HeuristicResult(f64, f64, f64, f64, f64);

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
impl Comparable for HeuristicResult {
    fn max() -> HeuristicResult {
        HeuristicResult(std::f64::INFINITY, 0.0, 0.0, 0.0, 0.0)
    }
    fn min() -> HeuristicResult {
        HeuristicResult(-std::f64::INFINITY, 0.0, 0.0, 0.0, 0.0)
    }
}
impl fmt::Debug for HeuristicResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:.2}, {:.2}, {:.2}, {:.2}, {:.2})",
            self.0, self.1, self.2, self.3, self.4
        )
    }
}

impl TreeAgent {
    pub fn new(_request: &GameRequest, config: &TreeConfig) -> TreeAgent {
        TreeAgent {
            config: config.clone(),
        }
    }

    fn heuristic(
        food: &[Vec2D],
        flood_fill: &mut FloodFill,
        game: &Game,
        turn: usize,
        config: &TreeConfig,
    ) -> HeuristicResult {
        match game.outcome() {
            Outcome::Match => HeuristicResult::default(),
            Outcome::Winner(0) => HeuristicResult::max(),
            Outcome::Winner(_) => HeuristicResult::min(),
            Outcome::None => {
                flood_fill.flood_snakes(&game.grid, &game.snakes, 0);
                let space = flood_fill.count_space_of(true);
                let mobility = space as f64 / (game.grid.width * game.grid.height) as f64;

                let health = game.snakes[0].health as f64 / 100.0;

                // Length advantage
                let own_len = game.snakes[0].body.len();
                let max_enemy_len = game.snakes[1..]
                    .iter()
                    .map(|s| s.body.len())
                    .max()
                    .unwrap_or(0);
                let len_advantage = own_len as f64 / max_enemy_len as f64;

                // Owned food
                let accessable_food =
                    food.iter().filter(|&&p| flood_fill[p].is_you()).count() as f64;
                let food_ownership = accessable_food / game.grid.width as f64;

                // Centrality
                let center_offset = game.snakes[0].head()
                    - Vec2D::new(game.grid.width as i16 / 2, game.grid.height as i16 / 2);
                let centrality = center_offset.x.abs().min(center_offset.y.abs()) as f64
                    / game.grid.width.max(game.grid.height) as f64;

                HeuristicResult(
                    mobility * config.mobility * (-(turn as f64) * config.mobility_decay).exp(),
                    health * config.health * (-(turn as f64) * config.health_decay).exp(),
                    len_advantage
                        * config.len_advantage
                        * (-(turn as f64) * config.len_advantage_decay).exp(),
                    food_ownership
                        * config.food_ownership
                        * (-(turn as f64) * config.food_ownership_decay).exp(),
                    centrality
                        * config.centrality
                        * (-(turn as f64) * config.centrality_decay).exp(),
                )
            }
        }
    }

    fn next_move(
        game: &Game,
        turn: usize,
        food: &[Vec2D],
        depth: usize,
        config: &TreeConfig,
    ) -> Option<Direction> {
        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);

        let start = Instant::now();
        let evaluation = max_n(&game, depth, |game| {
            TreeAgent::heuristic(&food, &mut flood_fill, game, turn + depth, config)
        });
        println!(
            "max_n {} {:?}ms {:?}",
            depth,
            (Instant::now() - start).as_millis(),
            evaluation
        );

        argmax(evaluation.iter())
            .filter(|&d| evaluation[d] > HeuristicResult::default())
            .map(|d| Direction::from(d as u8))
    }
}

impl Agent for TreeAgent {
    fn step(&mut self, request: &GameRequest, ms: u64) -> MoveResponse {
        let (sender, receiver) = mpsc::channel();
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
            2 => 5,
            3 => 3,
            _ => 2,
        };

        {
            let game = game.clone();
            let food = request.board.food.clone();
            let turn = request.turn;
            let config = self.config.clone();
            thread::spawn(move || {
                let result = TreeAgent::next_move(&game, turn, &food, depth, &config);
                if sender.send(result).is_err() {
                    println!("Timeout");
                }
            });
        };

        // Calculate the next move with smaller depth in case of timeouts
        let start = Instant::now();
        let alternative_result = TreeAgent::next_move(
            &game,
            request.turn,
            &request.board.food,
            depth - 1,
            &self.config,
        );
        let delta = Instant::now() - start;

        let result = receiver
            .recv_timeout(Duration::from_millis(ms) - delta)
            .ok()
            .flatten();

        if let Some(dir) = result {
            println!(">>> main: {:?}", dir);
            return MoveResponse::new(dir);
        }

        if let Some(dir) = alternative_result {
            println!(">>> alt: {:?}", dir);
            return MoveResponse::new(dir);
        }

        println!(">>> random");

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
        const COUNT: usize = 1000;

        let start = Instant::now();
        for _ in 0..COUNT {
            let d = agent.step(&game_req, 200);
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
