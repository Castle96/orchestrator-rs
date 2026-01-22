use crate::error::NetworkError;
use anyhow::Context;
use std::process::Command;
use tracing::info;

pub struct VlanManager;

impl VlanManager {
    /// Create a VLAN interface
    pub async fn create(
        parent: &str,
        vlan_id: u16,
        name: Option<&str>,
    ) -> Result<String, NetworkError> {
        let default_name = format!("{}.{}", parent, vlan_id);
        let vlan_name = name.unwrap_or(&default_name);
        info!("Creating VLAN {} on interface {}", vlan_id, parent);

        let output = Command::new("ip")
            .args([
                "link",
                "add",
                "link",
                parent,
                "name",
                vlan_name,
                "type",
                "vlan",
                "id",
                &vlan_id.to_string(),
            ])
            .output()
            .context("Failed to create VLAN")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        // Bring VLAN interface up
        let output = Command::new("ip")
            .args(["link", "set", vlan_name, "up"])
            .output()
            .context("Failed to bring VLAN up")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(vlan_name.to_string())
    }

    /// Delete a VLAN interface
    pub async fn delete(name: &str) -> Result<(), NetworkError> {
        info!("Deleting VLAN: {}", name);

        let output = Command::new("ip")
            .args(["link", "delete", name])
            .output()
            .context("Failed to delete VLAN")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }
}
