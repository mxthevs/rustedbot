use std::fs;
use std::process::Command;
use std::process::Stdio;
use uuid::Uuid;
use statrs::distribution::{Hypergeometric, DiscreteCDF};

use crate::database::sqlite;
use crate::helpers::{has_at_least_one_arg, has_more_than_one_arg, has_at_least_four_args};

pub enum BuiltinCommand {
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

                    let robocop_output = Command::new(robocop_path)
                        .arg(args)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .unwrap();

                    if !robocop_output.status.success() {
                        let response = String::from_utf8_lossy(&robocop_output.stderr);
                        log::error!("Robocop error: {response}");

                        let error_line = response.lines().nth(4).unwrap_or("Unknown error");
                        return String::from(error_line);
                    }

                    let uid = Uuid::new_v4();
                    let filename = format!("/tmp/{uid}.js");
                    let content = format!("console.log((() => ({args}))());");
                    fs::write(&filename, content).expect("Unable to write temporary JS file.");

                    let file = &filename;
                    let container_output = Command::new("docker")
                        .args([
                            "run",
                            "--rm",
                            "--memory",
                            "128m",
                            "--cpus",
                            "0.5",
                            "--network",
                            "none",
                            "--read-only",
                            "--pids-limit",
                            "64",
                            "-v",
                            format!("{file}:/sandbox/script.js:ro").as_str(),
                            "node-sandbox",
                            "node",
                            "/sandbox/script.js",
                        ])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output();

                    fs::remove_file(file).ok();

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
                            format!("USAGE: !odds <Deck Size> <Number of Successes> <Number of Draws> <Successes Needed>")
                        },
                    }
                }
                false => format!("USAGE: !odds <Deck Size> <Number of Successes> <Number of Draws> <Successes Needed>"),
            }
        }
    }
}
