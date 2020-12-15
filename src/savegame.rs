use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::io::Write;
use std::thread;

use crate::env::*;

pub fn worker(log_dir: PathBuf) -> Sender<Option<GameRequest>> {
    let (sender, receiver) = channel();

    if !log_dir.exists() {
        fs::create_dir(&log_dir).expect("Logging directory could not be created!");
    }

    thread::spawn(move || {
        println!("Start save worker {:?}", &log_dir);
        while let Some(game_req) = receiver.recv().unwrap() {
            save(game_req, &log_dir);
        }
        println!("Stopping save worker");
    });

    sender
}

fn save(game_req: GameRequest, log_dir: &Path) {
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
