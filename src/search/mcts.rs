//! # Monte Carlo Tree Search
//!
//! Idea: use mcts with a fast agent to simulate games instead of random playouts

use std::sync::Arc;

use log::{info, warn};
use mocats::UctPolicy;
use tokio::time::Instant;

use crate::{env::Direction, game::Game};

use super::Heuristic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Player(u8);

impl mocats::Player for Player {}
impl mocats::GameAction for Direction {}

#[derive(Debug, Clone)]
struct MctsGame {
    start: usize,
    game: Game,
    actions: Vec<Direction>,
    player: Player,
    heuristic: Arc<dyn Heuristic>,
}

impl mocats::GameState<Direction, Player> for MctsGame {
    fn get_actions(&self) -> Vec<Direction> {
        if self.game.turn > self.start + 8 {
            return Vec::new();
        }

        let mut moves: Vec<Direction> = self.game.valid_moves(self.player.0).collect();
        if self.player.0 != 0 && moves.is_empty() {
            moves.push(Direction::Up);
        }
        moves
    }

    fn apply_action(&mut self, action: &Direction) {
        self.actions.push(*action);
        if self.actions.len() == self.game.snakes.len() {
            info!("step={:?}", self.actions);
            self.game.step(&self.actions);
            self.actions.clear();
        }
        self.player = Player((self.player.0 + 1) % self.game.snakes.len() as u8);
    }

    fn get_turn(&self) -> Player {
        self.player
    }

    fn get_reward_for_player(&self, player: Player) -> f32 {
        let mut game = self.game.clone();
        game.snakes.swap(0, player.0 as usize);
        let res = self.heuristic.eval(&self.game) as f32;
        info!("reward={res} for {player:?}");
        res
    }
}

pub async fn mcts(heuristic: Arc<dyn Heuristic>, timeout: u64, game: &Game) -> Direction {
    let tree_policy = UctPolicy::new(2.0);

    let game = MctsGame {
        start: game.turn,
        game: game.clone(),
        actions: Vec::new(),
        player: Player(0),
        heuristic,
    };
    let mut search_tree = mocats::SearchTree::new(game, tree_policy);

    let start = Instant::now();
    while start.elapsed().as_millis() < timeout as _ {
        warn!(">>> mcts {:?}", start.elapsed().as_millis());
        async {
            search_tree.run(4);
        }
        .await;
    }

    search_tree.get_best_action().unwrap_or_default()
}

#[cfg(test)]
mod test {
    use crate::game::Game;
    use crate::logging;
    use crate::search::{mcts, Heuristic};
    use log::info;
    use std::sync::Arc;

    #[tokio::test]
    async fn simple() {
        logging();

        #[derive(Debug, Clone, Default)]
        struct SimpleHeuristic;
        impl Heuristic for SimpleHeuristic {
            fn eval(&self, game: &Game) -> f64 {
                if game.snake_is_alive(0) {
                    1.0
                } else {
                    0.0
                }
            }
        }

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . 0 . 1 . . . .
            . . . . ^ . ^ . . . .
            . . . . ^ . ^ . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . ."#,
        )
        .unwrap();

        let heuristic = Arc::new(SimpleHeuristic);
        let dir = mcts(heuristic, 1000, &game).await;
        info!("dir={:?}", dir);
    }
}
