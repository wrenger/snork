use std::path::Path;
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

use crate::env::*;

pub async fn save(game_req: GameRequest, log_dir: &Path) {
    if !log_dir.exists() {
        fs::create_dir(&log_dir)
            .await
            .expect("Logging directory could not be created!");
    }

    let filename = format!("{}.{}.json", game_req.game.id, game_req.you.id);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_dir.join(filename))
        .await
        .expect("Could not create/open save game!");

    let data = serde_json::to_vec(&game_req).unwrap();
    file.write_all(&data).await.unwrap();
}
