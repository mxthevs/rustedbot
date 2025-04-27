mod commands;
mod config;
mod helpers;
mod messages;
mod services;

use config::Config;
use fern::colors::{Color, ColoredLevelConfig};
use services::{database, twitch};
use std::process::ExitCode;

fn main() -> ExitCode {
    init_logger().expect("Failed to initialize logger.");
    log::info!("Starting RustedBot...");

    let config_path = std::env::args().nth(1);

    if config_path.is_none() {
        println!("Usage: rustedbot <config_path>");
        return ExitCode::FAILURE;
    }

    let config = Config::from_file(&config_path.unwrap());

    database::sqlite::migrate(config.trusted_users.clone());
    twitch::irc::init(config.user, config.token, config.channel, config.prefix);

    ExitCode::SUCCESS
}

pub fn init_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .debug(Color::Green)
        .info(Color::Blue)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                message
            ))
        })
        .chain(fern::log_file("rustedbot.log")?)
        .chain(std::io::stdout())
        .level(log::LevelFilter::Debug)
        .apply()?;

    Ok(())
}
