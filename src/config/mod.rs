use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub user: Option<String>,
    pub token: Option<String>,
    pub channel: String,
    pub prefix: String,
}

impl Config {
    pub fn from_file(path: &str) -> Config {
        let file = match fs::read_to_string(path) {
            Ok(file) => file,
            Err(_) => return default(),
        };

        let mut config = default();

        let loaded_config = file
            .split('\n')
            .into_iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| {
                if line.contains('#') {
                    let mut new_line = line;
                    new_line = &new_line[..new_line.find('#').unwrap()];
                    new_line
                } else {
                    line
                }
            })
            .map(|line| {
                line.split('=')
                    .into_iter()
                    .map(|part| part.trim())
                    .collect::<Vec<&str>>()
            })
            .map(|line| {
                if line[1].starts_with('"') && line[1].ends_with('"') {
                    let mut new_line = line.clone();
                    new_line[1] = &line[1][1..line[1].len() - 1];
                    new_line
                } else {
                    line
                }
            })
            .collect::<Vec<Vec<&str>>>();

        for line in loaded_config {
            match line[..] {
                ["twitch.user", user] => config.user = Some(String::from(user)),
                ["twitch.token", token] => {
                    let mut token = String::from(token);
                    if token.starts_with("oauth:") {
                        token = String::from(&token[6..]);
                    }
                    config.token = Some(token);
                }
                ["twitch.channel", channel] => config.channel = String::from(channel),
                ["command.prefix", prefix] => config.prefix = String::from(prefix),
                [unknown, _] => println!("Unknown config option: {unknown}"),
                _ => (),
            }
        }

        config
    }
}

pub fn default() -> Config {
    Config {
        user: None,
        token: None,
        channel: String::from("commanderroot"),
        prefix: String::from("!"),
    }
}
