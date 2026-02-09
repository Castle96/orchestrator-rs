use anyhow::Result;
use chrono::Utc;
use models::{StoragePool, StorageType};
use std::fs;
use std::path::Path;
use tracing::info;
use uuid::Uuid;

use crate::error::StorageError;

pub struct LocalStorageManager;

impl LocalStorageManager {
    /// Create a new local storage pool
    pub async fn create_pool(name: &str, path: &str) -> Result<StoragePool, StorageError> {
        info!("Creating local storage pool: {} at {}", name, path);

        let pool_path = Path::new(path);

        // Create directory if it doesn't exist
        fs::create_dir_all(pool_path).map_err(StorageError::Io)?;

        // Get filesystem statistics
        let (total_size, used_size) = Self::get_filesystem_stats(pool_path)?;
        let available_size = total_size.saturating_sub(used_size);

        Ok(StoragePool {
            id: Uuid::new_v4(),
            name: name.to_string(),
            storage_type: StorageType::Local,
            path: path.to_string(),
            total_size,
            used_size,
            available_size,
            created_at: Utc::now(),
        })
    }

    /// Get filesystem statistics for a path
    fn get_filesystem_stats(path: &Path) -> Result<(u64, u64), StorageError> {
        // This is a simplified version - in production, use statvfs or similar
        // For now, we'll use a basic approach
        let _metadata = fs::metadata(path).map_err(StorageError::Io)?;

        // Note: This doesn't give actual filesystem stats, but it's a placeholder
        // In production, you'd use libc::statvfs or similar
        let total_size = 100_000_000_000; // 100GB placeholder
        let used_size = Self::calculate_directory_size(path)?;

        Ok((total_size, used_size))
    }

    /// Calculate directory size recursively
    fn calculate_directory_size(path: &Path) -> Result<u64, StorageError> {
        let mut total = 0u64;

        if path.is_dir() {
            let entries = fs::read_dir(path).map_err(StorageError::Io)?;

            for entry in entries {
                let entry = entry.map_err(StorageError::Io)?;
                let path = entry.path();

                if path.is_dir() {
                    total += Self::calculate_directory_size(&path)?;
                } else {
                    total += entry.metadata().map_err(StorageError::Io)?.len();
                }
            }
        } else {
            total = fs::metadata(path).map_err(StorageError::Io)?.len();
        }

        Ok(total)
    }

    /// Delete a storage pool
    pub async fn delete_pool(path: &str) -> Result<(), StorageError> {
        info!("Deleting storage pool at: {}", path);

        if !Path::new(path).exists() {
            return Err(StorageError::PoolNotFound(path.to_string()));
        }

        fs::remove_dir_all(path).map_err(StorageError::Io)?;

        Ok(())
    }
}
