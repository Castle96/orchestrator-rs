use std::path::Path;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error};

use crate::lxc::LxcCommand;
use crate::config::LxcConfig;
use crate::error::ContainerError;
use models::{Container, ContainerStatus, ContainerConfig, CreateContainerRequest};

pub struct ContainerManager;

impl ContainerManager {
    /// Create a new container
    pub async fn create(request: CreateContainerRequest) -> Result<Container, ContainerError> {
        let container_id = Uuid::new_v4();
        let name = &request.name;

        // Check if container already exists
        if LxcCommand::exists(name) {
            return Err(ContainerError::AlreadyExists(name.to_string()));
        }

        info!("Creating container: {}", name);

        // Create container directory structure
        let container_dir = crate::config::LxcConfig::lxc_root().join(name);
        std::fs::create_dir_all(container_dir.join("rootfs"))
            .map_err(ContainerError::Io)?;

        // Write LXC configuration
        LxcConfig::write(name, &request.config)
            .map_err(|e| ContainerError::InvalidConfig(e.to_string()))?;

        // Create container using lxc-create
        // Note: This is a simplified version - in production, you'd need to handle templates
        // For now, we'll create a basic container structure
        // The actual lxc-create command format may vary by LXC version
        let create_result = LxcCommand::execute(&[
            "create",
            name,
            "-t",
            &request.template,
        ]);

        match create_result {
            Ok(_) => {
                info!("Container created successfully: {}", name);
                Ok(Container {
                    id: container_id,
                    name: name.clone(),
                    status: ContainerStatus::Stopped,
                    template: request.template,
                    node_id: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    config: request.config,
                })
            }
            Err(e) => {
                error!("Failed to create container: {}", e);
                Err(ContainerError::LxcCommandFailed(e.to_string()))
            }
        }
    }

    /// Start a container
    pub async fn start(name: &str) -> Result<(), ContainerError> {
        info!("Starting container: {}", name);
        
        if !LxcCommand::exists(name) {
            return Err(ContainerError::NotFound(name.to_string()));
        }

        LxcCommand::execute(&["start", name])
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        Ok(())
    }

    /// Stop a container
    pub async fn stop(name: &str) -> Result<(), ContainerError> {
        info!("Stopping container: {}", name);
        
        if !LxcCommand::exists(name) {
            return Err(ContainerError::NotFound(name.to_string()));
        }

        LxcCommand::execute(&["stop", name])
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        Ok(())
    }

    /// Delete a container
    pub async fn delete(name: &str) -> Result<(), ContainerError> {
        info!("Deleting container: {}", name);
        
        if !LxcCommand::exists(name) {
            return Err(ContainerError::NotFound(name.to_string()));
        }

        // Stop container first if running
        let _ = Self::stop(name).await;

        LxcCommand::execute(&["destroy", "-f", name])
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        Ok(())
    }

    /// Get container status
    pub async fn status(name: &str) -> Result<ContainerStatus, ContainerError> {
        if !LxcCommand::exists(name) {
            return Err(ContainerError::NotFound(name.to_string()));
        }

        let state = LxcCommand::state(name)
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))?;

        let status = match state.as_str() {
            "running" => ContainerStatus::Running,
            "stopped" => ContainerStatus::Stopped,
            "starting" => ContainerStatus::Starting,
            "stopping" => ContainerStatus::Stopping,
            "frozen" => ContainerStatus::Frozen,
            _ => ContainerStatus::Error,
        };

        Ok(status)
    }

    /// List all containers
    pub async fn list() -> Result<Vec<String>, ContainerError> {
        LxcCommand::list()
            .map_err(|e| ContainerError::LxcCommandFailed(e.to_string()))
    }

    /// Get container information
    pub async fn get(name: &str) -> Result<Container, ContainerError> {
        if !LxcCommand::exists(name) {
            return Err(ContainerError::NotFound(name.to_string()));
        }

        let status = Self::status(name).await?;
        let _config_str = LxcConfig::read(name)
            .map_err(|e| ContainerError::InvalidConfig(e.to_string()))?;

        // Parse config to get ContainerConfig
        // This is simplified - in production, you'd properly parse the LXC config
        let config = ContainerConfig {
            cpu_limit: None,
            memory_limit: None,
            disk_limit: None,
            network_interfaces: vec![],
            rootfs_path: format!("{}/rootfs", crate::config::LxcConfig::lxc_root().join(name).display()),
            environment: vec![],
        };

        Ok(Container {
            id: Uuid::new_v4(), // In production, store this in a database
            name: name.to_string(),
            status,
            template: "unknown".to_string(), // Parse from config
            node_id: None,
            created_at: Utc::now(), // Parse from filesystem
            updated_at: Utc::now(),
            config,
        })
    }
}
