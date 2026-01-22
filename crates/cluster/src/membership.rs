use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use tracing::info;
use models::{Node, NodeStatus, NodeResources};

pub struct MembershipManager {
    nodes: HashMap<Uuid, Node>,
    local_node_id: Uuid,
}

impl MembershipManager {
    pub fn new(local_node_id: Uuid) -> Self {
        Self {
            nodes: HashMap::new(),
            local_node_id,
        }
    }

    pub fn add_node(&mut self, node: Node) {
        info!("Adding node to cluster: {} ({})", node.name, node.id);
        self.nodes.insert(node.id, node);
    }

    pub fn remove_node(&mut self, node_id: &Uuid) {
        if let Some(node) = self.nodes.remove(node_id) {
            info!("Removing node from cluster: {} ({})", node.name, node.id);
        }
    }

    pub fn update_node_status(&mut self, node_id: &Uuid, status: NodeStatus) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.status = status;
            node.last_seen = Utc::now();
        }
    }

    pub fn update_node_resources(&mut self, node_id: &Uuid, resources: NodeResources) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.resources = resources;
        }
    }

    pub fn get_node(&self, node_id: &Uuid) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn list_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    pub fn get_local_node(&self) -> Option<&Node> {
        self.nodes.get(&self.local_node_id)
    }

    pub fn is_member(&self, node_id: &Uuid) -> bool {
        self.nodes.contains_key(node_id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
