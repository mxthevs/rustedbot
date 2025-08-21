use regex::Regex;
use statrs::distribution::{DiscreteCDF, Hypergeometric};
use std::fs;
use std::process::Command;
use std::process::Stdio;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tempfile::NamedTempFile;

use crate::database::sqlite;
use crate::helpers::{has_at_least_four_args, has_at_least_one_arg, has_more_than_one_arg};

#[derive(EnumIter)]
pub enum BuiltinCommand {
    Commands,
    Ping,
    AddCmd,
    DelCmd,
    UpdateCmd,
    Trust,
    Untrust,
    Wttr,
    GTASA,
    Node,
    Odds,
}

impl std::fmt::Display for BuiltinCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinCommand::Commands => write!(f, "commands"),
            BuiltinCommand::Ping => write!(f, "ping"),
            BuiltinCommand::AddCmd => write!(f, "addcmd"),
            BuiltinCommand::DelCmd => write!(f, "delcmd"),
            BuiltinCommand::UpdateCmd => write!(f, "updcmd"),
            BuiltinCommand::Trust => write!(f, "trust"),
            BuiltinCommand::Untrust => write!(f, "untrust"),
            BuiltinCommand::Wttr => write!(f, "clima"),
            BuiltinCommand::GTASA => write!(f, "gtasa"),
            BuiltinCommand::Node => write!(f, "node"),
            BuiltinCommand::Odds => write!(f, "odds"),
        }
    }
}

impl BuiltinCommand {
    pub fn from_string(command: &str) -> Option<BuiltinCommand> {
        match command {
            "commands" => Some(BuiltinCommand::Commands),
            "ping" => Some(BuiltinCommand::Ping),
            "addcmd" => Some(BuiltinCommand::AddCmd),
            "delcmd" => Some(BuiltinCommand::DelCmd),
            "updcmd" => Some(BuiltinCommand::UpdateCmd),
            "trust" => Some(BuiltinCommand::Trust),
            "untrust" => Some(BuiltinCommand::Untrust),
            "clima" => Some(BuiltinCommand::Wttr),
            "gtasa" => Some(BuiltinCommand::GTASA),
            "node" => Some(BuiltinCommand::Node),
            "odds" => Some(BuiltinCommand::Odds),
            _ => None,
        }
    }

    pub fn requires_trust(&self) -> bool {
        matches!(
            self,
            BuiltinCommand::AddCmd
                | BuiltinCommand::DelCmd
                | BuiltinCommand::UpdateCmd
                | BuiltinCommand::Trust
                | BuiltinCommand::Untrust
                | BuiltinCommand::Node
        )
    }

    pub async fn execute(&self, args: &str, sender: &str) -> String {
        if self.requires_trust() && !sqlite::is_trusted(sender) {
            log::warn!("User {sender} tried to run the \"{self}\" command without permission. Consider adding them to the trusted users list.");
            return format!("@{sender} you are not authorized to run this command.");
        }

        match self {
            BuiltinCommand::Ping => String::from("Pong!"),
            BuiltinCommand::Commands => {
                let commands_in_db = sqlite::get_commands();

                if let Err(e) = commands_in_db {
                    log::error!("Failed to retrieve commands: {e}");
                    return format!("Error retrieving commands.");
                }

                let commands_in_db = commands_in_db.unwrap()
                .iter()
                .map(|(name, _)| format!("{name}"))
                .collect::<Vec<_>>()
                .join(", ");

                let builtin_commands: BuiltinCommandIter = BuiltinCommand::iter();
                let mut commands = String::new();

                for command in builtin_commands {
                    commands.push_str(&format!("{command}, "));
                }

                let mut commands = commands.trim().to_string();

                while commands.ends_with(",") {
                    commands.pop();
                }

                let all_commands = if commands_in_db.len() > 0 {
                    format!("{commands}, {commands_in_db}")
                } else {
                    commands
                };

                String::from(all_commands)
            },
            BuiltinCommand::AddCmd => match has_more_than_one_arg(args) {
                true => {
                    let mut parts = args.splitn(2, ' ');
                    let name = parts.next().unwrap_or("");
                    let response = parts.next().unwrap_or("").trim();

                    let re = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
                    if !re.is_match(name) {
                        log::error!("{sender} tried to add an invalid command name: {name}");
                        return String::from("Command name can only contain letters and numbers.");
                    }

                    let existing_command = BuiltinCommand::iter().find(|cmd| format!("{cmd}") == name);
                    if existing_command.is_some() {
                        log::error!("{sender} tried to add a command that already exists as a built-in command: {name}");
                        return String::from("The command you are trying to add already exists as a built-in command.");
                    }

                    let command_in_db = sqlite::get_command_response(name);
                    if command_in_db.is_ok() {
                       sqlite::update_command_response(name, response);
                       log::info!("{sender} tried to add a command that already exists: {name}. It was updated with {response}");
                       return String::from("Command already exists. It was updated with the new response.");
                    }

                    sqlite::create_command(name, response);
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
            BuiltinCommand::Trust => match has_at_least_one_arg(args) {
                true => {
                    let name = args.split(' ').collect::<Vec<&str>>()[0];
                    sqlite::trust_user(name);
                    format!("{name} is now trusted.")
                }
                false => format!("@{sender} USAGE: trust <name>"),
            },
            BuiltinCommand::Untrust => match has_at_least_one_arg(args) {
                true => {
                    let name = args.split(' ').collect::<Vec<&str>>()[0];
                    sqlite::untrust_user(name);
                    format!("{name} is now untrusted.")
                }
                false => format!("@{sender} USAGE: untrust <name>"),
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
            BuiltinCommand::GTASA => {
                let file = match fs::read_to_string("data/main.scm") {
                    Ok(file) => file,
                    Err(_) => return String::from("Error reading file"),
                };

                let lines = file
                    .split('\n')
                    .into_iter()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<&str>>();

                let line = match has_at_least_one_arg(args) {
                    true => {
                        let target = args.split(' ').collect::<Vec<&str>>()[0];

                        let mut lines_with_target = Vec::new();

                        for l in lines.iter() {
                            let target = target.to_ascii_lowercase();
                            if l.to_ascii_lowercase().contains(target.as_str()) {
                                lines_with_target.push(l);
                            }
                        }

                        if lines_with_target.is_empty() {
                            return format!("");
                        }

                        let line = rand::random::<usize>() % lines_with_target.len();

                        lines_with_target[line]
                    }
                    false => {
                        let line = rand::random::<usize>() % lines.len();
                        lines[line]
                    }
                };

                format!("{line}")
            }
            BuiltinCommand::Node => match has_at_least_one_arg(args) {
                true => {
                    let robocop_path = "vendor/robocop/bin";

                    if let Err(_) = fs::metadata(robocop_path) {
                        log::error!("Robocop not found. Please run `make build` to install it.");
                        return String::from("Security measures not found. Not continuing.");
                    }

                    let docker_system_info = Command::new("docker")
                        .arg("system")
                        .arg("info")
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();

                    let is_docker_running = matches!(docker_system_info, Ok(s) if s.success());

                    if !is_docker_running {
                        log::error!("User {sender} tried to run the node command but Docker is not running.");
                        return String::from("Docker is not running. Not continuing.");
                    }

                    let robocop_output = Command::new(robocop_path)
                        .arg(args)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .unwrap();

                    if !robocop_output.status.success() {
                        const KNOWN_ROBOCOP_EXCEPTIONS: [&str; 2] = ["InfiniteLoopError", "ForbiddenModuleError"];

                        let response = String::from_utf8_lossy(&robocop_output.stderr);
                        let error_line = response.lines().nth(4).unwrap_or("Unknown error");

                        if let Some(exception) = KNOWN_ROBOCOP_EXCEPTIONS.iter().find(|e| error_line.starts_with(*e)) {
                            log::warn!("User {sender} triggered the {exception} security measure with `{args}`");
                        } else {
                            log::error!("User {sender} triggered a Robocop exception: {response} with `{args}`");
                        }

                        return String::from(error_line);
                    }

                    let file = NamedTempFile::new().expect("Unable to create temporary JS file.");
                    let content = format!("console.log((() => ({args}))());");
                    fs::write(file.path(), content).expect("Unable to write temporary JS file.");

                    let container_output = Command::new("docker")
                        .args([
                            "run", "--rm", "--memory", "128m", "--cpus", "0.5",
                            "--network", "none", "--read-only", "--pids-limit", "64",
                            "-v", &format!("{}:/sandbox/script.js:ro", file.path().display()),
                            "node-sandbox", "node", "/sandbox/script.js",
                        ])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output();

                    match container_output {
                        Ok(output) => {
                            if output.status.success() {
                                String::from_utf8_lossy(&output.stdout).to_string()
                            } else {
                                let error_message = String::from_utf8_lossy(&output.stderr);
                                log::error!("Error while running JavaScript code: {error_message}");

                                let error_line = error_message
                                    .lines()
                                    .nth(4)
                                    .unwrap_or("Unknown error")
                                    .to_string();

                                error_line
                            }
                        }
                        Err(e) => {
                            log::error!("Error running Docker container: {e}");
                            String::from("There was an error while trying to run the code.")
                        }
                    }
                }
                false => format!("@{sender} USAGE: node <code>"),
            },
            BuiltinCommand::Odds => match has_at_least_four_args(args) {
                true => {
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    let deck_size = parts[0].parse::<u64>();
                    let num_successes = parts[1].parse::<u64>();
                    let num_draws = parts[2].parse::<u64>();
                    let min_successes = parts[3].parse::<u64>();

                    match (deck_size, num_successes, num_draws, min_successes) {
                        (Ok(deck_size), Ok(num_successes), Ok(num_draws), Ok(min_successes)) => {
                            if num_successes > deck_size || num_draws > deck_size {
                                return format!("Error: More successes or draws than cards in deck.");
                            }

                            let hyper = Hypergeometric::new(num_successes, deck_size - num_successes, num_draws);
                            if let Ok(h) = hyper {
                                let prob = 1.0 - h.cdf(min_successes - 1);
                                let percentage = prob * 100.0;

                                format!("Odds of drawing {min_successes} or more of {num_successes} cards from {num_draws} draws in a {deck_size} card deck: {percentage:.5}%")
                            } else {
                                log::error!("Error creating hypergeometric distribution: {args}");
                                format!("Error creating hypergeometric distribution.")
                            }
                        }
                        _ => {
                            log::error!("Invalid parameters for hypergeometric calculation: {args}");
                            format!("USAGE: odds <Deck Size> <Number of Successes> <Number of Draws> <Successes Needed>")
                        },
                    }
                }
                false => format!("USAGE: odds <Deck Size> <Number of Successes> <Number of Draws> <Successes Needed>"),
            }
        }
    }
}
