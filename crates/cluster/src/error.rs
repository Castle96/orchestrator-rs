use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClusterError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Cluster operation failed: {0}")]
    OperationFailed(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
