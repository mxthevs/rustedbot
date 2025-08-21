use super::Command;
use crate::helpers::has_at_least_one_arg;
use crate::register_command;

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use std::process::Command as TerminalCommand;
use std::process::Stdio;
use tempfile::NamedTempFile;

#[derive(Default)]
pub struct Node;

#[async_trait]
impl Command for Node {
    fn name(&self) -> &'static str {
        "node"
    }

    fn requires_trust(&self) -> bool {
        true
    }

    async fn execute(&self, args: &str, sender: &str) -> String {
        match run_node(args, sender).await {
            Ok(output) => output,
            Err(e) => {
                log::error!("Execution error: {e}");
                e.to_string()
            }
        }
    }
}

async fn run_node(args: &str, sender: &str) -> Result<String> {
    if !has_at_least_one_arg(args) {
        anyhow::bail!("USAGE: node <code>");
    }

    check_robocop_installed("vendor/robocop/bin")?;
    ensure_docker_running(sender).await?;

    run_robocop(args, sender).await?;

    let js_file = create_temp_js(args)?;
    run_in_docker(&js_file, sender).await
}

fn check_robocop_installed(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        log::error!("Robocop not found at {path}. Please run `make build` to install it.");
        anyhow::bail!("Security measures not found. Not continuing.");
    }
    Ok(())
}

async fn ensure_docker_running(sender: &str) -> Result<()> {
    let status = TerminalCommand::new("docker")
        .args(["system", "info"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to check Docker status")?;

    if !status.success() {
        log::error!("User {sender} tried to run the node command but Docker is not running.");
        anyhow::bail!("Docker is not running. Not continuing.");
    }

    Ok(())
}

async fn run_robocop(args: &str, sender: &str) -> Result<()> {
    let output = TerminalCommand::new("vendor/robocop/bin")
        .arg(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run Robocop")?;

    if !output.status.success() {
        const KNOWN_EXCEPTIONS: [&str; 2] = ["InfiniteLoopError", "ForbiddenModuleError"];
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_line = stderr.lines().nth(4).unwrap_or("Unknown error");

        if KNOWN_EXCEPTIONS.iter().any(|e| error_line.starts_with(e)) {
            log::warn!("User {sender} triggered a security measure: `{error_line}` with `{args}`");
        } else {
            log::error!("User {sender} triggered a Robocop exception: {stderr} with `{args}`");
        }

        anyhow::bail!(error_line.to_string());
    }

    Ok(())
}

fn create_temp_js(args: &str) -> Result<NamedTempFile> {
    let file = NamedTempFile::new().context("Unable to create temporary JS file")?;
    let content = format!("console.log((() => ({args}))());");
    fs::write(file.path(), content).context("Unable to write temporary JS file")?;
    Ok(file)
}

async fn run_in_docker(js_file: &NamedTempFile, _sender: &str) -> Result<String> {
    let output = TerminalCommand::new("docker")
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
            &format!("{}:/sandbox/script.js:ro", js_file.path().display()),
            "node-sandbox",
            "node",
            "/sandbox/script.js",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Error running Docker container")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("Error while running JavaScript code: {stderr}");
        let error_line = stderr.lines().nth(4).unwrap_or("Unknown error");
        anyhow::bail!(error_line.to_string())
    }
}

register_command!(Node);
