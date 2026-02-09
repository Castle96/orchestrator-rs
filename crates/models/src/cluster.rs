use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: Uuid,
    pub name: String,
    pub nodes: Vec<Uuid>,
    pub leader_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub cluster: Cluster,
    pub node_count: usize,
    pub total_resources: ClusterResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResources {
    pub total_cpu_cores: u32,
    pub total_memory: u64, // in bytes
    pub total_disk: u64,   // in bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatusResponse {
    pub cluster: ClusterInfo,
}
