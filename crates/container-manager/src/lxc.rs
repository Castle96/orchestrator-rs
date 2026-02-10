use anyhow::{Context, Result};
use std::process::Command;
use tracing::{debug, error, warn};

pub struct LxcCommand;

impl LxcCommand {
    /// Check if running as root
    fn is_root() -> bool {
        nix::unistd::getuid().is_root()
    }

    /// Execute an LXC command with smart privilege escalation
    pub fn execute(args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("No command specified"));
        }

        let cmd_name = format!("lxc-{}", args[0]);
        debug!("Executing: {}", cmd_name);

        // Try direct execution first (works if running as root)
        if Self::is_root() {
            return Self::execute_direct(&cmd_name, &args[1..]);
        }

        // Try with passwordless sudo
        match Self::execute_with_sudo(&cmd_name, &args[1..]) {
            Ok(output) => Ok(output),
            Err(e) => {
                warn!("Sudo execution failed: {}", e);
                Err(anyhow::anyhow!("LXC operations require root privileges. Please run the orchestrator as root or configure passwordless sudo for LXC commands. Error: {}", e))
            }
        }
    }

    /// Execute command directly (when running as root)
    fn execute_direct(cmd_name: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(cmd_name)
            .args(args)
            .output()
            .context(format!("Failed to execute LXC command: {}", cmd_name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("LXC command failed: {}", stderr);
            return Err(anyhow::anyhow!("LXC command failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Execute command with sudo (assumes passwordless sudo configured)
    fn execute_with_sudo(cmd_name: &str, args: &[&str]) -> Result<String> {
        let output = Command::new("sudo")
            .arg("-n") // non-interactive mode
            .arg(cmd_name)
            .args(args)
            .output()
            .context(format!(
                "Failed to execute LXC command with sudo: {}",
                cmd_name
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("LXC command with sudo failed: {}", stderr);

            if stderr.contains("sudo: a password is required") {
                return Err(anyhow::anyhow!(
                    "Passwordless sudo not configured for LXC commands"
                ));
            }

            return Err(anyhow::anyhow!("LXC command with sudo failed: {}", stderr));
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
        let output = Self::execute(&["ls", "--line"])?;
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
