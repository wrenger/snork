use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::env::*;

pub async fn save(game_req: GameRequest, log_dir: &Path) {
    if !log_dir.exists() {
        fs::create_dir(&log_dir).expect("Logging directory could not be created!");
    }

    let filename = format!("{}.{}.json", game_req.game.id, game_req.you.id);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_dir.join(filename))
        .expect("Could not create/open save game!");
    serde_json::to_writer(&mut file, &game_req).unwrap();
    writeln!(file).unwrap();
}
