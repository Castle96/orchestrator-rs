use std::process::Command;
use anyhow::Context;
use tracing::{info, warn};
use crate::error::NetworkError;

pub struct FirewallManager;

impl FirewallManager {
    /// Add an iptables rule
    pub async fn add_rule(
        chain: &str,
        rule: &[&str],
    ) -> Result<(), NetworkError> {
        info!("Adding iptables rule to chain {}: {:?}", chain, rule);

        let mut args = vec!["-A", chain];
        args.extend(rule);

        let output = Command::new("iptables")
            .args(&args)
            .output()
            .context("Failed to execute iptables command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to add iptables rule: {}", stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Delete an iptables rule
    pub async fn delete_rule(
        chain: &str,
        rule: &[&str],
    ) -> Result<(), NetworkError> {
        info!("Deleting iptables rule from chain {}: {:?}", chain, rule);

        let mut args = vec!["-D", chain];
        args.extend(rule);

        let output = Command::new("iptables")
            .args(&args)
            .output()
            .context("Failed to execute iptables command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Allow traffic from a container interface
    pub async fn allow_container_interface(interface: &str) -> Result<(), NetworkError> {
        Self::add_rule("FORWARD", &["-i", interface, "-j", "ACCEPT"]).await?;
        Self::add_rule("FORWARD", &["-o", interface, "-j", "ACCEPT"]).await?;
        Ok(())
    }

    /// Block traffic from a container interface
    pub async fn block_container_interface(interface: &str) -> Result<(), NetworkError> {
        Self::delete_rule("FORWARD", &["-i", interface, "-j", "ACCEPT"]).await?;
        Self::delete_rule("FORWARD", &["-o", interface, "-j", "ACCEPT"]).await?;
        Ok(())
    }
}
