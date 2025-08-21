use super::Command;
use crate::commands::registry::is_builtin;
use crate::database::sqlite;
use crate::helpers::has_at_least_n_args;
use crate::register_command;

use regex::Regex;

use async_trait::async_trait;

#[derive(Default)]
pub struct UpdateCmd;

#[async_trait]
impl Command for UpdateCmd {
    fn name(&self) -> &'static str {
        "updcmd"
    }

    fn requires_trust(&self) -> bool {
        true
    }

    async fn execute(&self, args: &str, sender: &str) -> String {
        match parse_args(args, sender) {
            Ok((name, response)) => handle_command(name, response, sender),
            Err(msg) => msg,
        }
    }
}

fn parse_args<'a>(args: &'a str, sender: &str) -> Result<(&'a str, &'a str), String> {
    if !has_at_least_n_args(args, 2) {
        return Err(format!("@{sender} USAGE: updcmd <name> <response>"));
    }

    let mut parts = args.splitn(2, ' ');
    let name = parts.next().unwrap_or("");
    let response = parts.next().unwrap_or("").trim();

    if !is_valid_name(name) {
        log::error!("{sender} tried to update an invalid command name: {name}");
        return Err("Command name can only contain letters and numbers.".to_string());
    }

    Ok((name, response))
}

fn is_valid_name(name: &str) -> bool {
    Regex::new(r"^[a-zA-Z0-9]+$").unwrap().is_match(name)
}

fn handle_command(name: &str, response: &str, sender: &str) -> String {
    if is_builtin(name) {
        log::error!(
            "{sender} tried to update a command that already exists as a built-in command: {name}"
        );

        return String::from(
            "The command you are trying to update already exists as a built-in command.",
        );
    }

    sqlite::update_command_response(name, response);
    String::from("Command updated!")
}

register_command!(UpdateCmd);
