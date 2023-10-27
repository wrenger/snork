use env_logger::fmt::Formatter;
use owo_colors::{AnsiColors, OwoColorize};
use std::io::Write;

// Exported to be accessable in benchmarks
pub mod agents;
pub mod env;
pub mod floodfill;
pub mod game;
pub mod grid;
mod savegame;
pub mod search;
mod util;

pub fn logging() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .is_test(cfg!(test))
        .format(logging_format)
        .try_init();
}

fn logging_format(buf: &mut Formatter, record: &log::Record) -> std::io::Result<()> {
    let color = match record.level() {
        log::Level::Error => AnsiColors::BrightRed,
        log::Level::Warn => AnsiColors::BrightYellow,
        log::Level::Info => AnsiColors::BrightBlack,
        log::Level::Debug => AnsiColors::BrightBlack,
        log::Level::Trace => AnsiColors::BrightBlack,
    };

    writeln!(
        buf,
        "{}",
        format_args!(
            "[{:5} {}:{}] {}",
            record.level(),
            record.file().unwrap_or_default(),
            record.line().unwrap_or_default(),
            record.args()
        )
        .color(color)
    )
}
