use crate::env::*;
use crate::game::Game;
use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RandomAgent;

impl RandomAgent {
    pub async fn step(&self, game: &Game) -> MoveResponse {
        let mut rng = SmallRng::from_entropy();
        MoveResponse::new(
            game.valid_moves(0)
                .choose(&mut rng)
                .unwrap_or(Direction::Up),
        )
    }
}
