use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub cluster_id: Uuid,
    pub leader_id: Option<Uuid>,
    pub node_assignments: HashMap<Uuid, Vec<Uuid>>, // node_id -> container_ids
    pub storage_allocations: HashMap<Uuid, Vec<Uuid>>, // pool_id -> volume_ids
}

impl ClusterState {
    pub fn new(cluster_id: Uuid) -> Self {
        Self {
            cluster_id,
            leader_id: None,
            node_assignments: HashMap::new(),
            storage_allocations: HashMap::new(),
        }
    }

    pub fn set_leader(&mut self, leader_id: Uuid) {
        info!("Setting cluster leader: {}", leader_id);
        self.leader_id = Some(leader_id);
    }

    pub fn assign_container(&mut self, node_id: Uuid, container_id: Uuid) {
        debug!("Assigning container {} to node {}", container_id, node_id);
        self.node_assignments
            .entry(node_id)
            .or_default()
            .push(container_id);
    }

    pub fn unassign_container(&mut self, node_id: &Uuid, container_id: &Uuid) {
        if let Some(containers) = self.node_assignments.get_mut(node_id) {
            containers.retain(|id| id != container_id);
        }
    }

    pub fn get_node_containers(&self, node_id: &Uuid) -> Vec<Uuid> {
        self.node_assignments
            .get(node_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn allocate_storage(&mut self, pool_id: Uuid, volume_id: Uuid) {
        debug!("Allocating volume {} to pool {}", volume_id, pool_id);
        self.storage_allocations
            .entry(pool_id)
            .or_default()
            .push(volume_id);
    }
}
