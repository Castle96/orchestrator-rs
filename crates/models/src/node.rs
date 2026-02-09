use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub status: NodeStatus,
    pub cluster_id: Option<Uuid>,
    pub resources: NodeResources,
    pub joined_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Online,
    Offline,
    Joining,
    Leaving,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu_cores: u32,
    pub memory_total: u64, // in bytes
    pub memory_used: u64,  // in bytes
    pub disk_total: u64,   // in bytes
    pub disk_used: u64,    // in bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeListResponse {
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinClusterRequest {
    pub cluster_name: String,
    pub node_address: String,
    pub node_port: u16,
}
