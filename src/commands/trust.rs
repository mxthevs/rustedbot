use super::Command;
use crate::database::sqlite;
use crate::helpers::has_at_least_n_args;
use crate::register_command;

use regex::Regex;

use async_trait::async_trait;

#[derive(Default)]
pub struct Trust;

#[async_trait]
impl Command for Trust {
    fn name(&self) -> &'static str {
        "trust"
    }

    fn requires_trust(&self) -> bool {
        true
    }

    async fn execute(&self, args: &str, sender: &str) -> String {
        match parse_args(args, sender) {
            Ok(name) => handle_command(name, sender),
            Err(msg) => msg,
        }
    }
}

fn parse_args<'a>(args: &'a str, sender: &str) -> Result<&'a str, String> {
    if !has_at_least_n_args(args, 1) {
        return Err(format!("@{sender} USAGE: trust <username>"));
    }

    let username: &str = args.split(' ').collect::<Vec<&str>>()[0];

    if !is_valid_name(username) {
        log::error!("{sender} tried to trust an invalid username: {username}");
        return Err("Username must be a valid Twitch username.".to_string());
    }

    Ok(username)
}

fn is_valid_name(name: &str) -> bool {
    Regex::new("^[a-z][a-z0-9_]{2,24}$").unwrap().is_match(name)
}

fn handle_command(username: &str, sender: &str) -> String {
    sqlite::trust_user(username);
    log::info!("{sender} trusted user: {username}");

    format!("{username} has been trusted.")
}

register_command!(Trust);
