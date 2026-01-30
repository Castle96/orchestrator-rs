/// Container snapshot management
use anyhow::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

use crate::error::ContainerError;
use crate::lxc::LxcCommand;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub container_name: String,
    pub name: String,
    pub comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub size_bytes: Option<u64>,
}

pub struct SnapshotManager;

impl SnapshotManager {
    /// Create a snapshot of a container
    pub async fn create(
        container_name: &str,
        snapshot_name: Option<String>,
        comment: Option<String>,
    ) -> Result<Snapshot, ContainerError> {
        // Verify container exists
        if !LxcCommand::exists(container_name) {
            return Err(ContainerError::NotFound(container_name.to_string()));
        }

        // Generate snapshot name if not provided
        let snap_name = snapshot_name.unwrap_or_else(|| {
            format!("snap_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"))
        });

        info!(
            "Creating snapshot '{}' for container '{}'",
            snap_name, container_name
        );

        // Use lxc-snapshot to create the snapshot
        let args = if let Some(ref c) = comment {
            vec!["snapshot", "-n", &snap_name, "-c", c, container_name]
        } else {
            vec!["snapshot", "-n", &snap_name, container_name]
        };

        LxcCommand::execute(&args)
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        // Get snapshot size
        let snapshot_path = Self::get_snapshot_path(container_name, &snap_name);
        let size_bytes = Self::get_directory_size(&snapshot_path).ok();

        Ok(Snapshot {
            id: Uuid::new_v4(),
            container_name: container_name.to_string(),
            name: snap_name,
            comment,
            created_at: Utc::now(),
            size_bytes,
        })
    }

    /// List all snapshots for a container
    pub async fn list(container_name: &str) -> Result<Vec<Snapshot>, ContainerError> {
        if !LxcCommand::exists(container_name) {
            return Err(ContainerError::NotFound(container_name.to_string()));
        }

        info!("Listing snapshots for container '{}'", container_name);

        // Use lxc-snapshot to list snapshots
        let output = LxcCommand::execute(&["snapshot", "-L", container_name])
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        let mut snapshots = Vec::new();

        // Parse the snapshot list output
        // LXC snapshot output format varies, but typically shows snapshot names
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("List of") || line.starts_with("---") {
                continue;
            }

            // Parse snapshot name (first word in the line)
            if let Some(snap_name) = line.split_whitespace().next() {
                let snapshot_path = Self::get_snapshot_path(container_name, snap_name);
                let size_bytes = Self::get_directory_size(&snapshot_path).ok();

                snapshots.push(Snapshot {
                    id: Uuid::new_v4(),
                    container_name: container_name.to_string(),
                    name: snap_name.to_string(),
                    comment: None,
                    created_at: Utc::now(), // Would need to parse from metadata
                    size_bytes,
                });
            }
        }

        Ok(snapshots)
    }

    /// Restore a container from a snapshot
    pub async fn restore(
        container_name: &str,
        snapshot_name: &str,
    ) -> Result<(), ContainerError> {
        if !LxcCommand::exists(container_name) {
            return Err(ContainerError::NotFound(container_name.to_string()));
        }

        info!(
            "Restoring container '{}' from snapshot '{}'",
            container_name, snapshot_name
        );

        // Use lxc-snapshot to restore
        LxcCommand::execute(&["snapshot", "-r", snapshot_name, container_name])
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        Ok(())
    }

    /// Delete a snapshot
    pub async fn delete(
        container_name: &str,
        snapshot_name: &str,
    ) -> Result<(), ContainerError> {
        if !LxcCommand::exists(container_name) {
            return Err(ContainerError::NotFound(container_name.to_string()));
        }

        info!(
            "Deleting snapshot '{}' for container '{}'",
            snapshot_name, container_name
        );

        // Use lxc-snapshot to delete
        LxcCommand::execute(&["snapshot", "-d", snapshot_name, container_name])
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        Ok(())
    }

    /// Clone a container from a snapshot
    pub async fn clone(
        source_container: &str,
        snapshot_name: &str,
        new_container_name: &str,
    ) -> Result<(), ContainerError> {
        if !LxcCommand::exists(source_container) {
            return Err(ContainerError::NotFound(source_container.to_string()));
        }

        if LxcCommand::exists(new_container_name) {
            return Err(ContainerError::AlreadyExists(new_container_name.to_string()));
        }

        info!(
            "Cloning container '{}' from snapshot '{}' to '{}'",
            source_container, snapshot_name, new_container_name
        );

        // Use lxc-copy to clone from snapshot
        // Format: lxc-copy -n source -s snapshot -N new_name
        LxcCommand::execute(&[
            "copy",
            "-n",
            source_container,
            "-s",
            snapshot_name,
            "-N",
            new_container_name,
        ])
        .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        Ok(())
    }

    /// Get the path to a snapshot directory
    fn get_snapshot_path(container_name: &str, snapshot_name: &str) -> PathBuf {
        crate::config::LxcConfig::lxc_root().as_path()
            .join(container_name)
            .join("snaps")
            .join(snapshot_name)
    }

    /// Calculate the size of a directory recursively
    fn get_directory_size(path: &Path) -> Result<u64> {
        let mut size = 0u64;

        if !path.exists() {
            return Ok(0);
        }

        if path.is_file() {
            return Ok(path.metadata()?.len());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                size += metadata.len();
            } else if metadata.is_dir() {
                size += Self::get_directory_size(&entry.path())?;
            }
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_path() {
        let path = SnapshotManager::get_snapshot_path("test-container", "snap1");
        assert_eq!(
            path,
            crate::config::LxcConfig::lxc_root().join("test-container").join("snaps").join("snap1")
        );
    }

    #[test]
    fn test_snapshot_name_generation() {
        let name = format!("snap_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        assert!(name.starts_with("snap_"));
        assert!(name.len() > 5);
    }
}
