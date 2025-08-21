use super::Command;
use crate::database::sqlite;
use crate::helpers::has_more_than_one_arg;
use crate::register_command;

use regex::Regex;

use async_trait::async_trait;

#[derive(Default)]
pub struct Untrust;

#[async_trait]
impl Command for Untrust {
    fn name(&self) -> &'static str {
        "untrust"
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
    if !has_more_than_one_arg(args) {
        return Err(format!("@{sender} USAGE: untrust <username>"));
    }

    let username: &str = args.split(' ').collect::<Vec<&str>>()[0];

    if !is_valid_name(username) {
        log::error!("{sender} tried to untrust an invalid username: {username}");
        return Err("Username must be a valid Twitch username.".to_string());
    }

    Ok(username)
}

fn is_valid_name(name: &str) -> bool {
    Regex::new("^[a-z][a-z0-9_]{2,24}$").unwrap().is_match(name)
}

fn handle_command(username: &str, sender: &str) -> String {
    sqlite::untrust_user(username);
    log::info!("{sender} untrusted user: {username}");

    format!("{username} has been untrusted.")
}

register_command!(Untrust);
