use crate::error::StorageError;
use anyhow::Result;
use chrono::Utc;
use models::Volume;
use std::fs;
use std::path::Path;
use tracing::info;
use uuid::Uuid;

pub struct VolumeManager;

impl VolumeManager {
    /// Create a new volume in a storage pool
    pub async fn create_volume(
        pool_path: &str,
        name: &str,
        size: u64,
    ) -> Result<Volume, StorageError> {
        info!(
            "Creating volume: {} in pool {} (size: {} bytes)",
            name, pool_path, size
        );

        let volume_path = Path::new(pool_path).join(name);

        // Create volume directory
        fs::create_dir_all(&volume_path).map_err(StorageError::Io)?;

        // In production, you might create a sparse file or use other volume management
        // For now, we'll just create the directory

        Ok(Volume {
            id: Uuid::new_v4(),
            name: name.to_string(),
            pool_id: Uuid::new_v4(), // In production, get from pool
            size,
            used: 0,
            created_at: Utc::now(),
        })
    }

    /// Delete a volume
    pub async fn delete_volume(pool_path: &str, name: &str) -> Result<(), StorageError> {
        info!("Deleting volume: {} from pool {}", name, pool_path);

        let volume_path = Path::new(pool_path).join(name);

        if !volume_path.exists() {
            return Err(StorageError::VolumeNotFound(name.to_string()));
        }

        fs::remove_dir_all(&volume_path).map_err(StorageError::Io)?;

        Ok(())
    }

    /// Get volume information
    pub async fn get_volume(pool_path: &str, name: &str) -> Result<Volume, StorageError> {
        let volume_path = Path::new(pool_path).join(name);

        if !volume_path.exists() {
            return Err(StorageError::VolumeNotFound(name.to_string()));
        }

        // Calculate used space
        let used = Self::calculate_directory_size(&volume_path)?;

        Ok(Volume {
            id: Uuid::new_v4(), // In production, get from database
            name: name.to_string(),
            pool_id: Uuid::new_v4(),
            size: 0, // In production, get from metadata
            used,
            created_at: Utc::now(),
        })
    }

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
}
