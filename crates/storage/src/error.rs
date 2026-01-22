use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Storage pool not found: {0}")]
    PoolNotFound(String),

    #[error("Volume not found: {0}")]
    VolumeNotFound(String),

    #[error("Storage operation failed: {0}")]
    OperationFailed(String),

    #[error("Insufficient space: requested {0}, available {1}")]
    InsufficientSpace(u64, u64),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
