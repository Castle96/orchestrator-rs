use std::process::Command;
use anyhow::{Context, Result};
use tracing::{info, error};
use models::{Bridge, CreateBridgeRequest};
use crate::error::NetworkError;

pub struct BridgeManager;

impl BridgeManager {
    /// Create a new Linux bridge
    pub async fn create(request: CreateBridgeRequest) -> Result<Bridge, NetworkError> {
        info!("Creating bridge: {}", request.name);

        // Check if bridge already exists
        if Self::exists(&request.name)? {
            return Err(NetworkError::BridgeExists(request.name));
        }

        // Create bridge using ip command
        let output = Command::new("ip")
            .args(["link", "add", "name", &request.name, "type", "bridge"])
            .output()
            .context("Failed to execute ip command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to create bridge: {}", stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        // Set STP if requested
        if request.stp_enabled {
            let output = Command::new("ip")
                .args(["link", "set", &request.name, "type", "bridge", "stp", "on"])
                .output()
                .context("Failed to set STP")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Failed to enable STP: {}", stderr);
            }
        }

        // Bring bridge up
        Self::set_up(&request.name).await?;

        // Set IP address if provided
        if let Some(ref ip) = request.ip_address {
            Self::set_ip(&request.name, ip).await?;
        }

        Ok(Bridge {
            name: request.name,
            interfaces: vec![],
            ip_address: request.ip_address,
            stp_enabled: request.stp_enabled,
        })
    }

    /// Delete a bridge
    pub async fn delete(name: &str) -> Result<(), NetworkError> {
        info!("Deleting bridge: {}", name);

        if !Self::exists(name)? {
            return Err(NetworkError::InterfaceNotFound(name.to_string()));
        }

        // Bring bridge down first
        let _ = Self::set_down(name).await;

        // Delete bridge
        let output = Command::new("ip")
            .args(["link", "delete", name])
            .output()
            .context("Failed to delete bridge")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Check if bridge exists
    pub fn exists(name: &str) -> Result<bool, NetworkError> {
        let output = Command::new("ip")
            .args(["link", "show", name])
            .output()
            .context("Failed to check bridge")?;

        Ok(output.status.success())
    }

    /// List all bridges
    pub async fn list() -> Result<Vec<String>, NetworkError> {
        let output = Command::new("ip")
            .args(["-br", "link", "show", "type", "bridge"])
            .output()
            .context("Failed to list bridges")?;

        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let bridges: Vec<String> = stdout
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .map(|s| s.to_string())
            .collect();

        Ok(bridges)
    }

    /// Add interface to bridge
    pub async fn add_interface(bridge: &str, interface: &str) -> Result<(), NetworkError> {
        info!("Adding interface {} to bridge {}", interface, bridge);

        let output = Command::new("ip")
            .args(["link", "set", interface, "master", bridge])
            .output()
            .context("Failed to add interface to bridge")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Remove interface from bridge
    pub async fn remove_interface(bridge: &str, interface: &str) -> Result<(), NetworkError> {
        info!("Removing interface {} from bridge {}", interface, bridge);

        let output = Command::new("ip")
            .args(["link", "set", interface, "nomaster"])
            .output()
            .context("Failed to remove interface from bridge")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    async fn set_up(name: &str) -> Result<(), NetworkError> {
        let output = Command::new("ip")
            .args(["link", "set", name, "up"])
            .output()
            .context("Failed to bring interface up")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    async fn set_down(name: &str) -> Result<(), NetworkError> {
        let output = Command::new("ip")
            .args(["link", "set", name, "down"])
            .output()
            .context("Failed to bring interface down")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    async fn set_ip(name: &str, ip: &str) -> Result<(), NetworkError> {
        let output = Command::new("ip")
            .args(["addr", "add", ip, "dev", name])
            .output()
            .context("Failed to set IP address")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }
}
