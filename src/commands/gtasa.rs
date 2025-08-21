use super::Command;
use crate::helpers::has_at_least_one_arg;
use crate::register_command;

use async_trait::async_trait;
use rand::Rng;
use std::fs;

#[derive(Default)]
pub struct Gta;

#[async_trait]
impl Command for Gta {
    fn name(&self) -> &'static str {
        "gta"
    }

    async fn execute(&self, args: &str, _sender: &str) -> String {
        let lines = match read_lines_from_file("data/main.scm") {
            Ok(lines) => lines,
            Err(_) => return String::from("Error reading file"),
        };

        let target = if has_at_least_one_arg(args) {
            Some(args.split_whitespace().next().unwrap())
        } else {
            None
        };

        match choose_line(&lines, target) {
            Some(line) => String::from(line),
            None => String::from(""),
        }
    }
}

fn read_lines_from_file(path: &str) -> Result<Vec<String>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let lines = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    Ok(lines)
}

fn choose_line<'a>(lines: &'a [String], target: Option<&str>) -> Option<&'a str> {
    let filtered: Vec<&str> = match target {
        Some(t) => {
            let t = t.to_ascii_lowercase();
            lines
                .iter()
                .map(|line| line.as_str())
                .filter(|line| line.to_ascii_lowercase().contains(&t))
                .collect()
        }
        None => lines.iter().map(|line| line.as_str()).collect(),
    };

    if filtered.is_empty() {
        None
    } else {
        let index = rand::thread_rng().gen_range(0..filtered.len());
        Some(filtered[index])
    }
}

register_command!(Gta);
