use super::Command;
use crate::commands::registry::Registry;
use crate::database::sqlite;
use crate::register_command;

use async_trait::async_trait;

#[derive(Default)]
pub struct Commands;

#[async_trait]
impl Command for Commands {
    fn name(&self) -> &'static str {
        "commands"
    }

    async fn execute(&self, _args: &str, _sender: &str) -> String {
        let db_commands = fetch_db_commands();
        let builtin_commands = format_builtin_commands();

        combine_commands(builtin_commands, db_commands)
    }
}

fn fetch_db_commands() -> String {
    match sqlite::get_commands() {
        Ok(commands) => commands
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>()
            .join(", "),
        Err(e) => {
            log::error!("Failed to retrieve commands: {e}");
            String::new()
        }
    }
}

fn format_builtin_commands() -> String {
    Registry::all()
        .iter()
        .filter(|cmd| !cmd.requires_trust())
        .map(|cmd| cmd.name().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn combine_commands(builtin: String, db: String) -> String {
    match (builtin.is_empty(), db.is_empty()) {
        (false, false) => format!("{builtin}, {db}"),
        (false, true) => builtin,
        (true, false) => db,
        (true, true) => String::new(),
    }
}

register_command!(Commands);
