use std::process::Command;
use std::process::Stdio;

use crate::database::sqlite;
use crate::helpers::{has_at_least_one_arg, has_more_than_one_arg};

pub enum BuiltinCommand {
    Ping,
    AddCmd,
    DelCmd,
    UpdateCmd,
    Wttr,
    Node,
}

impl BuiltinCommand {
    pub fn from_string(command: &str) -> Option<BuiltinCommand> {
        match command {
            "ping" => Some(BuiltinCommand::Ping),
            "addcmd" => Some(BuiltinCommand::AddCmd),
            "delcmd" => Some(BuiltinCommand::DelCmd),
            "updcmd" => Some(BuiltinCommand::UpdateCmd),
            "clima" => Some(BuiltinCommand::Wttr),
            "node" => Some(BuiltinCommand::Node),
            _ => None,
        }
    }

    // args is a string with all the content after the command
    // each command can parse it as it wants
    pub async fn execute(&self, args: &str, sender: &str) -> String {
        match self {
            BuiltinCommand::Ping => String::from("Pong!"),
            BuiltinCommand::AddCmd => match has_more_than_one_arg(args) {
                true => {
                    let name = args.split(' ').collect::<Vec<&str>>()[0];
                    let response = args.split(' ').collect::<Vec<&str>>()[1..].join(" ");

                    sqlite::create_command(name, response.as_str());
                    String::from("Command added!")
                }
                false => format!("@{sender} USAGE: addcmd <name> <response>"),
            },
            BuiltinCommand::DelCmd => match has_at_least_one_arg(args) {
                true => {
                    let name = args.split(' ').collect::<Vec<&str>>()[0];
                    sqlite::delete_command(name);
                    String::from("Command deleted!")
                }
                false => format!("@{sender} USAGE: delcmd <name>"),
            },
            BuiltinCommand::UpdateCmd => match has_more_than_one_arg(args) {
                true => {
                    let name = args.split(' ').collect::<Vec<&str>>()[0];
                    let response = args.split(' ').collect::<Vec<&str>>()[1..].join(" ");

                    sqlite::update_command_response(name, response.as_str());
                    String::from("Command updated!")
                }
                false => format!("@{sender} USAGE: updcmd <name> <response>"),
            },
            BuiltinCommand::Wttr => match has_at_least_one_arg(args) {
                true => {
                    let format = r#"%l: %c 🌡️%t\n"#;
                    let args = args.replace(" ", "+");
                    let url = format!("https://wttr.in/{args}?format={format}&m");
                    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
                    body
                }
                false => format!("@{sender} USAGE: clima <local>"),
            },
            BuiltinCommand::Node => match has_at_least_one_arg(args) {
                true => {
                    let command = format!("require('./vendor/robocop/index.js').run(`{args}`)");
                    let output = Command::new("node")
                        .args(["-e", format!("console.log({command})").as_str()])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .unwrap();

                    let response = match output.status.success() {
                        true => String::from_utf8(output.stdout).unwrap(),
                        false => {
                            let response = String::from_utf8(output.stderr).unwrap();
                            let error = response.split('\n').collect::<Vec<&str>>()[4];
                            String::from(error)
                        }
                    };

                    response
                }
                false => format!("@{sender} USAGE: node <code>"),
            },
        }
    }
}
