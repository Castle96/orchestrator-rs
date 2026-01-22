use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Interface not found: {0}")]
    InterfaceNotFound(String),

    #[error("Bridge already exists: {0}")]
    BridgeExists(String),

    #[error("Network operation failed: {0}")]
    OperationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}
