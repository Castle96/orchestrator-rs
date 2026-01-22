use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: Uuid,
    pub name: String,
    pub status: ContainerStatus,
    pub template: String,
    pub node_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config: ContainerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Frozen,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub cpu_limit: Option<u32>,
    pub memory_limit: Option<u64>, // in bytes
    pub disk_limit: Option<u64>,   // in bytes
    pub network_interfaces: Vec<ContainerNetworkInterface>,
    pub rootfs_path: String,
    pub environment: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerNetworkInterface {
    pub name: String,
    pub bridge: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContainerRequest {
    pub name: String,
    pub template: String,
    pub config: ContainerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResponse {
    pub container: Container,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerListResponse {
    pub containers: Vec<Container>,
}
