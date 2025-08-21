use super::Command;
use crate::commands::registry::is_builtin;
use crate::database::sqlite;
use crate::helpers::has_at_least_n_args;
use crate::register_command;

use regex::Regex;

use async_trait::async_trait;

#[derive(Default)]
pub struct DelCmd;

#[async_trait]
impl Command for DelCmd {
    fn name(&self) -> &'static str {
        "delcmd"
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
        return Err(format!("@{sender} USAGE: delcmd <name>"));
    }

    let name: &str = args.split(' ').collect::<Vec<&str>>()[0];

    if !is_valid_name(name) {
        log::error!("{sender} tried to delete an invalid command name: {name}");
        return Err("Command name can only contain letters and numbers.".to_string());
    }

    Ok(name)
}

fn is_valid_name(name: &str) -> bool {
    Regex::new(r"^[a-zA-Z0-9]+$").unwrap().is_match(name)
}

fn handle_command(name: &str, sender: &str) -> String {
    if is_builtin(name) {
        log::error!("{sender} tried to delete a built-in command: {name}");
        return String::from("The command you are trying to delete already is a built-in command.");
    }

    sqlite::delete_command(name);
    String::from("Command deleted!")
}

register_command!(DelCmd);
