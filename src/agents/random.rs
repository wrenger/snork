use crate::{env::*, game::Game};
use rand::seq::IteratorRandom;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RandomAgent;

impl RandomAgent {
    pub async fn step(&self, game: &Game) -> MoveResponse {
        let mut rng = rand::thread_rng();
        MoveResponse::new(
            game.valid_moves(0)
                .choose(&mut rng)
                .unwrap_or(Direction::Up),
        )
    }
}
