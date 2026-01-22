pub mod error;
pub mod local;
pub mod shared;
pub mod volumes;

pub use error::*;
pub use local::*;
pub use shared::*;
pub use volumes::*;

#[cfg(test)]
mod tests {
    
    use models::{CreateStoragePoolRequest, StorageType};

    #[tokio::test]
    async fn test_storage_pool_creation_request_validation() {
        let request = CreateStoragePoolRequest {
            name: "test-pool".to_string(),
            storage_type: StorageType::Local,
            path: "/var/lib/storage/test-pool".to_string(),
        };

        assert_eq!(request.name, "test-pool");
        assert_eq!(request.storage_type, StorageType::Local);
        assert!(request.path.starts_with("/var/lib/storage"));
    }

    #[test]
    fn test_storage_pool_name_validation() {
        let valid_names = vec!["pool1", "storage-pool", "test123", "vm-storage"];
        for name in valid_names {
            assert!(
                is_valid_pool_name(name),
                "Pool name '{}' should be valid",
                name
            );
        }

        let invalid_names = vec![
            "",
            "Pool1",
            "pool_name",
            "pool name",
            "pool.",
            ".pool",
            "pool-name-that-is-way-too-long-for-storage-pools-and-exceeds-reasonable-limits",
        ];
        for name in invalid_names {
            assert!(
                !is_valid_pool_name(name),
                "Pool name '{}' should be invalid",
                name
            );
        }
    }

    #[test]
    fn test_storage_path_validation() {
        let valid_paths = vec!["/var/lib/storage", "/mnt/storage", "/opt/storage/pools"];
        for path in valid_paths {
            assert!(
                is_valid_storage_path(path),
                "Storage path '{}' should be valid",
                path
            );
        }

        let invalid_paths = vec![
            "",
            "relative/path",
            "/tmp", // tmp is usually not suitable for storage
            "/proc/something",
            "/sys/something",
        ];
        for path in invalid_paths {
            assert!(
                !is_valid_storage_path(path),
                "Storage path '{}' should be invalid",
                path
            );
        }
    }

    #[test]
    fn test_storage_type_serialization() {
        use serde_json;

        let local = StorageType::Local;
        let nfs = StorageType::Nfs;
        let cifs = StorageType::Cifs;

        let local_json = serde_json::to_string(&local).unwrap();
        let nfs_json = serde_json::to_string(&nfs).unwrap();
        let cifs_json = serde_json::to_string(&cifs).unwrap();

        assert_eq!(local_json, "\"local\"");
        assert_eq!(nfs_json, "\"nfs\"");
        assert_eq!(cifs_json, "\"cifs\"");
    }

    #[test]
    fn test_nfs_path_parsing() {
        let valid_nfs_paths = vec![
            "192.168.1.100:/exports/storage",
            "storage.example.com:/data",
            "nfs-server:/var/nfs/storage",
        ];

        for path in valid_nfs_paths {
            let parsed = parse_nfs_path(path);
            assert!(
                parsed.is_some(),
                "NFS path '{}' should parse correctly",
                path
            );
            let (server, export) = parsed.unwrap();
            assert!(!server.is_empty());
            assert!(!export.is_empty());
            assert!(export.starts_with('/'));
        }

        let invalid_nfs_paths = vec!["invalid-path", "/local/path", "server-only", ":export-only"];

        for path in invalid_nfs_paths {
            let parsed = parse_nfs_path(path);
            assert!(parsed.is_none(), "NFS path '{}' should not parse", path);
        }
    }

    #[test]
    fn test_cifs_path_parsing() {
        let valid_cifs_paths = vec![
            "//192.168.1.100/share",
            "//server.example.com/storage",
            "//fileserver/data",
        ];

        for path in valid_cifs_paths {
            let parsed = parse_cifs_path(path);
            assert!(
                parsed.is_some(),
                "CIFS path '{}' should parse correctly",
                path
            );
            let (server, share) = parsed.unwrap();
            assert!(!server.is_empty());
            assert!(!share.is_empty());
        }

        let invalid_cifs_paths = vec!["invalid-path", "/local/path", "//server-only", "//server/"];

        for path in invalid_cifs_paths {
            let parsed = parse_cifs_path(path);
            assert!(parsed.is_none(), "CIFS path '{}' should not parse", path);
        }
    }

    // Helper functions for validation
    fn is_valid_pool_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 32 {
            return false;
        }

        // Must start with letter or number
        if !name.chars().next().unwrap_or('_').is_ascii_alphanumeric() {
            return false;
        }

        // Can only contain lowercase letters, numbers, and hyphens
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    fn is_valid_storage_path(path: &str) -> bool {
        if path.is_empty() {
            return false;
        }

        // Must be absolute path
        if !path.starts_with('/') {
            return false;
        }

        // Avoid problematic paths
        let forbidden_prefixes = ["/tmp", "/proc", "/sys", "/dev"];
        for prefix in &forbidden_prefixes {
            if path.starts_with(prefix) {
                return false;
            }
        }

        true
    }

    fn parse_nfs_path(path: &str) -> Option<(String, String)> {
        if let Some(colon_pos) = path.find(':') {
            let server = path[..colon_pos].to_string();
            let export = path[colon_pos + 1..].to_string();

            if !server.is_empty() && !export.is_empty() && export.starts_with('/') {
                return Some((server, export));
            }
        }
        None
    }

    fn parse_cifs_path(path: &str) -> Option<(String, String)> {
        if let Some(path_without_prefix) = path.strip_prefix("//") {
            if let Some(slash_pos) = path_without_prefix.find('/') {
                let server = path_without_prefix[..slash_pos].to_string();
                let share = path_without_prefix[slash_pos + 1..].to_string();

                if !server.is_empty() && !share.is_empty() {
                    return Some((server, share));
                }
            }
        }
        None
    }
}
