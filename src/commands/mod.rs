pub enum BuiltinCommand {
    Ping,
}

impl BuiltinCommand {
    pub fn from_string(command: &str) -> Option<BuiltinCommand> {
        match command {
            "ping" => Some(BuiltinCommand::Ping),
            _ => None,
        }
    }

    // args is a string with all the content after the command
    // each command can parse it as it wants
    pub fn execute(&self, _args: &str) -> String {
        match self {
            BuiltinCommand::Ping => String::from("Pong!"),
        }
    }
}
