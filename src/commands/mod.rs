use crate::database::sqlite;

pub enum BuiltinCommand {
    Ping,
    AddCmd,
    DelCmd,
    UpdateCmd,
}

impl BuiltinCommand {
    pub fn from_string(command: &str) -> Option<BuiltinCommand> {
        match command {
            "ping" => Some(BuiltinCommand::Ping),
            "addcmd" => Some(BuiltinCommand::AddCmd),
            "delcmd" => Some(BuiltinCommand::DelCmd),
            "updcmd" => Some(BuiltinCommand::UpdateCmd),
            _ => None,
        }
    }

    // args is a string with all the content after the command
    // each command can parse it as it wants
    pub fn execute(&self, args: &str) -> String {
        match self {
            BuiltinCommand::Ping => String::from("Pong!"),
            BuiltinCommand::AddCmd => match args.split(' ').collect::<Vec<&str>>().len() > 1 {
                true => {
                    let name = args.split(' ').collect::<Vec<&str>>()[0];
                    let response = args.split(' ').collect::<Vec<&str>>()[1..].join(" ");

                    sqlite::create_command(name, response.as_str());
                    String::from("Command added!")
                }
                false => String::from("USAGE addcmd <name> <response>"),
            },
            BuiltinCommand::DelCmd => match args.split(' ').collect::<Vec<&str>>().len() > 0 {
                true => {
                    let name = args.split(' ').collect::<Vec<&str>>()[0];
                    sqlite::delete_command(name);
                    String::from("Command deleted!")
                }
                false => String::from("USAGE delcmd <name>"),
            },
            BuiltinCommand::UpdateCmd => match args.split(' ').collect::<Vec<&str>>().len() > 1 {
                true => {
                    let name = args.split(' ').collect::<Vec<&str>>()[0];
                    let response = args.split(' ').collect::<Vec<&str>>()[1..].join(" ");

                    sqlite::update_command_response(name, response.as_str());
                    String::from("Command updated!")
                }
                false => String::from("USAGE updcmd <name> <response>"),
            },
        }
    }
}
