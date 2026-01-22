use anyhow::{Context, Result};
use std::process::Command;
use tracing::{debug, error};

pub struct LxcCommand;

impl LxcCommand {
    /// Execute an LXC command and return stdout
    pub fn execute(args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("No command specified"));
        }

        debug!("Executing: lxc-{}", args.join(" "));

        let cmd_name = format!("lxc-{}", args[0]);
        let output = Command::new(&cmd_name)
            .args(&args[1..])
            .output()
            .context(format!("Failed to execute LXC command: {}", cmd_name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("LXC command failed: {}", stderr);
            return Err(anyhow::anyhow!("LXC command failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Check if a container exists
    pub fn exists(name: &str) -> bool {
        Self::list()
            .unwrap_or_default()
            .iter()
            .any(|container| container == name)
    }

    /// List all containers
    pub fn list() -> Result<Vec<String>> {
        let output = Self::execute(&["list", "-1", "-n"])?;
        Ok(output
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// Get container state
    pub fn state(name: &str) -> Result<String> {
        let output = Self::execute(&["info", name])?;
        // Parse state from info output
        for line in output.lines() {
            if line.starts_with("State:") {
                return Ok(line.split(':').nth(1).unwrap_or("").trim().to_lowercase());
            }
        }
        Err(anyhow::anyhow!("Could not parse container state"))
    }
}
