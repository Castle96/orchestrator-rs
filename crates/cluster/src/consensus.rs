use crate::error::ClusterError;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: Vec<u8>,
}

pub struct RaftNode {
    pub node_id: Uuid,
    pub state: RaftState,
    pub current_term: u64,
    pub voted_for: Option<Uuid>,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub next_index: HashMap<Uuid, u64>,
    pub match_index: HashMap<Uuid, u64>,
}

impl RaftNode {
    pub fn new(node_id: Uuid) -> Self {
        Self {
            node_id,
            state: RaftState::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }

    pub fn become_candidate(&mut self) -> Result<(), ClusterError> {
        info!(
            "Node {} becoming candidate for term {}",
            self.node_id,
            self.current_term + 1
        );
        self.current_term += 1;
        self.state = RaftState::Candidate;
        self.voted_for = Some(self.node_id);
        Ok(())
    }

    pub fn become_leader(&mut self) -> Result<(), ClusterError> {
        info!(
            "Node {} becoming leader for term {}",
            self.node_id, self.current_term
        );
        self.state = RaftState::Leader;
        Ok(())
    }

    pub fn become_follower(&mut self, term: u64) {
        if term > self.current_term {
            info!("Node {} becoming follower for term {}", self.node_id, term);
            self.current_term = term;
            self.state = RaftState::Follower;
            self.voted_for = None;
        }
    }

    pub fn append_entry(&mut self, entry: LogEntry) {
        self.log.push(entry);
    }

    pub fn commit(&mut self, index: u64) {
        if index > self.commit_index {
            self.commit_index = index;
        }
    }
}
