use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePool {
    pub id: Uuid,
    pub name: String,
    pub storage_type: StorageType,
    pub path: String,
    pub total_size: u64,     // in bytes
    pub used_size: u64,      // in bytes
    pub available_size: u64, // in bytes
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    Local,
    Nfs,
    Cifs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: Uuid,
    pub name: String,
    pub pool_id: Uuid,
    pub size: u64, // in bytes
    pub used: u64, // in bytes
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePoolListResponse {
    pub pools: Vec<StoragePool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStoragePoolRequest {
    pub name: String,
    pub storage_type: StorageType,
    pub path: String,
}
