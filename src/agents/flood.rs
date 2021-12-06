use std::time::Duration;
use std::time::Instant;

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::env::*;
use crate::game::{async_max_n, FloodFill, Game};
use crate::util::argmax;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FloodAgent {
    board_control: f64,
    health: f64,
    len_advantage: f64,
    food_distance: f64,
}

impl Default for FloodAgent {
    fn default() -> Self {
        Self {
            board_control: 2.0,
            health: 0.5,
            len_advantage: 0.5,
            food_distance: 0.5,
        }
    }
}

impl FloodAgent {
    pub async fn step(&self, request: &GameRequest, ms: u64) -> MoveResponse {
        let mut game = Game::new(request.board.width, request.board.height);
        game.reset_from_request(&request);

        let (sender, mut receiver) = mpsc::channel(32);

        let _ = tokio::time::timeout(
            Duration::from_millis(ms),
            Self::iterative_tree_search(&self, &game, sender),
        )
        .await;

        let mut result = None;
        while let Some(dir) = receiver.recv().await {
            result = Some(dir);
        }

        if let Some(dir) = result {
            return MoveResponse::new(Direction::from(dir as u8));
        }

        println!(">>> none");
        MoveResponse::new(game.valid_moves(0).next().unwrap_or(Direction::Up))
    }

    async fn iterative_tree_search(&self, game: &Game, sender: Sender<Direction>) {
        // Iterative deepening
        for depth in 1..8 {
            let (dir, value) = self.next_move(game, depth).await;

            // Stop and fallback to random possible move
            if value <= f64::MIN {
                break;
            }

            if sender.send(dir).await.is_err()
                // Terminate if we probably win/lose
                || value >= f64::MAX
            {
                break;
            }
        }
    }

    /// Performes a tree search and returns the maximized heuristic and move.
    pub async fn next_move(&self, game: &Game, depth: usize) -> (Direction, f64) {
        let start = Instant::now();

        let config = self.clone();
        let result = async_max_n(&game, depth, move |game| config.heuristic(game)).await;

        println!(
            ">>> max_n {} {:?}ms {:?}",
            depth,
            start.elapsed().as_millis(),
            result
        );

        argmax(result.iter())
            .map(|d| (Direction::from(d as u8), result[d]))
            .unwrap_or_default()
    }

    pub fn heuristic(&self, game: &Game) -> f64 {
        if game.snake_is_alive(0) {
            let own_len = game.snakes[0].body.len() as f64;
            let area = (game.grid.width * game.grid.height) as f64;

            let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
            let food_distances = flood_fill.flood_snakes(&game.grid, &game.snakes);
            let food_distance = food_distances
                .into_iter()
                .filter(|&d| d < u16::MAX)
                .map(|d| (area - d as f64) / area)
                .sum::<f64>();

            let board_control = flood_fill.count_health(true) as f64 / (area * 100.0);

            let health = game.snakes[0].health as f64 / 100.0;

            let max_enemy_len = game.snakes[1..]
                .iter()
                .map(|s| s.body.len())
                .max()
                .unwrap_or(0) as f64;
            let len_advantage = (own_len + food_distance * self.food_distance) - max_enemy_len;

            self.board_control * board_control
                + self.health * health
                + self.len_advantage * len_advantage
        } else {
            -f64::INFINITY
        }
    }
}
