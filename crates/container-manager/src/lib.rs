pub mod config;
pub mod container;
pub mod error;
pub mod lxc;
pub mod snapshot;

pub use container::*;
pub use error::*;
pub use snapshot::*;

#[cfg(test)]
mod tests {

    use models::{ContainerConfig, ContainerNetworkInterface, CreateContainerRequest};

    #[tokio::test]
    async fn test_container_creation_request_validation() {
        let request = CreateContainerRequest {
            name: "test-container".to_string(),
            template: "alpine".to_string(),
            config: ContainerConfig {
                cpu_limit: Some(2),
                memory_limit: Some(1024 * 1024 * 1024), // 1GB
                disk_limit: Some(10 * 1024 * 1024 * 1024), // 10GB
                network_interfaces: vec![ContainerNetworkInterface {
                    name: "eth0".to_string(),
                    bridge: "lxcbr0".to_string(),
                    ipv4: Some("192.168.1.100/24".to_string()),
                    ipv6: None,
                    mac: None,
                }],
                rootfs_path: "/var/lib/lxc/test-container/rootfs".to_string(),
                environment: vec![
                    ("USER".to_string(), "root".to_string()),
                    ("HOME".to_string(), "/root".to_string()),
                ],
            },
        };

        assert_eq!(request.name, "test-container");
        assert_eq!(request.template, "alpine");
        assert!(request.config.cpu_limit.is_some());
        assert!(request.config.memory_limit.is_some());
    }

    #[tokio::test]
    async fn test_container_name_validation() {
        // Valid container names
        let valid_names = vec!["test", "test-container", "test123", "web-server"];
        for name in valid_names {
            assert!(
                is_valid_container_name(name),
                "Name '{}' should be valid",
                name
            );
        }

        // Invalid container names
        let invalid_names = vec![
            "",
            "Test",
            "test_container",
            "test container",
            "test.",
            ".test",
        ];
        for name in invalid_names {
            assert!(
                !is_valid_container_name(name),
                "Name '{}' should be invalid",
                name
            );
        }
    }

    #[test]
    fn test_lxc_command_parsing() {
        let command = ["list", "-1"];
        assert_eq!(command[0], "list");
        assert_eq!(command[1], "-1");
    }

    #[test]
    fn test_container_state_parsing() {
        let states = vec![
            ("running", models::ContainerStatus::Running),
            ("stopped", models::ContainerStatus::Stopped),
            ("starting", models::ContainerStatus::Starting),
            ("stopping", models::ContainerStatus::Stopping),
            ("frozen", models::ContainerStatus::Frozen),
            ("unknown", models::ContainerStatus::Error),
        ];

        for (state_str, expected_status) in states {
            let parsed = parse_container_state(state_str);
            assert_eq!(parsed, expected_status);
        }
    }

    // Helper function for container name validation
    fn is_valid_container_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 64 {
            return false;
        }

        // Must start with alphanumeric
        if !name.chars().next().unwrap_or('_').is_ascii_alphanumeric() {
            return false;
        }

        // Can only contain lowercase letters, numbers, and hyphens
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    // Helper function for parsing container states
    fn parse_container_state(state: &str) -> models::ContainerStatus {
        match state.to_lowercase().as_str() {
            "running" => models::ContainerStatus::Running,
            "stopped" => models::ContainerStatus::Stopped,
            "starting" => models::ContainerStatus::Starting,
            "stopping" => models::ContainerStatus::Stopping,
            "frozen" => models::ContainerStatus::Frozen,
            _ => models::ContainerStatus::Error,
        }
    }
}
