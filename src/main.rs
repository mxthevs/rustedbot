mod commands;
mod config;
mod helpers;
mod services;

use config::Config;
use services::{database, twitch};
use std::process::ExitCode;

fn main() -> ExitCode {
    let config_path = std::env::args().nth(1);

    if config_path.is_none() {
        println!("Usage: rustedbot <config_path>");
        return ExitCode::FAILURE;
    }

    let config = Config::from_file(&config_path.unwrap());

    database::sqlite::migrate();
    twitch::irc::init(config.user, config.token, config.channel, config.prefix);

    ExitCode::SUCCESS
}
