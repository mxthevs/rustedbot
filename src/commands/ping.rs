use super::Command;
use crate::register_command;

use async_trait::async_trait;

#[derive(Default)]
pub struct Ping;

#[async_trait]
impl Command for Ping {
    fn name(&self) -> &'static str {
        "ping"
    }

    async fn execute(&self, _args: &str, _sender: &str) -> String {
        String::from("Pong!")
    }
}

register_command!(Ping);
