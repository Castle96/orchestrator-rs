pub mod bridge;
pub mod error;
pub mod firewall;
pub mod vlan;

pub use bridge::*;
pub use error::*;
pub use firewall::*;
pub use vlan::*;

#[cfg(test)]
mod tests {
    
    use models::CreateBridgeRequest;

    #[tokio::test]
    async fn test_bridge_creation_request_validation() {
        let request = CreateBridgeRequest {
            name: "test-bridge".to_string(),
            ip_address: Some("192.168.1.1/24".to_string()),
            stp_enabled: true,
        };

        assert_eq!(request.name, "test-bridge");
        assert!(request.ip_address.is_some());
        assert!(request.stp_enabled);
    }

    #[test]
    fn test_bridge_name_validation() {
        let valid_names = vec!["br0", "lxcbr0", "docker0", "test-bridge"];
        for name in valid_names {
            assert!(
                is_valid_bridge_name(name),
                "Bridge name '{}' should be valid",
                name
            );
        }

        let invalid_names = vec![
            "",
            "br0.",
            ".br0",
            "br 0",
            "BR0",
            "bridge-name-that-is-way-too-long-for-linux-interfaces",
        ];
        for name in invalid_names {
            assert!(
                !is_valid_bridge_name(name),
                "Bridge name '{}' should be invalid",
                name
            );
        }
    }

    #[test]
    fn test_ip_address_validation() {
        let valid_ips = vec![
            "192.168.1.1/24",
            "10.0.0.1/8",
            "172.16.0.1/16",
            "fd00::1/64",
        ];
        for ip in valid_ips {
            assert!(
                is_valid_ip_address(ip),
                "IP address '{}' should be valid",
                ip
            );
        }

        let invalid_ips = vec![
            "192.168.1.1",
            "192.168.1.256/24",
            "not.an.ip/24",
            "192.168.1.1/33",
        ];
        for ip in invalid_ips {
            assert!(
                !is_valid_ip_address(ip),
                "IP address '{}' should be invalid",
                ip
            );
        }
    }

    #[test]
    fn test_vlan_id_validation() {
        // Valid VLAN IDs (1-4094)
        assert!(is_valid_vlan_id(1));
        assert!(is_valid_vlan_id(100));
        assert!(is_valid_vlan_id(4094));

        // Invalid VLAN IDs
        assert!(!is_valid_vlan_id(0));
        assert!(!is_valid_vlan_id(4095));
        assert!(!is_valid_vlan_id(4095));
    }

    // Helper functions for validation
    fn is_valid_bridge_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 15 {
            return false;
        }

        // Must start with letter
        if !name.chars().next().unwrap_or('_').is_ascii_alphabetic() {
            return false;
        }

        // Can only contain lowercase letters, numbers, and hyphens
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    fn is_valid_ip_address(ip: &str) -> bool {
        if !ip.contains('/') {
            return false;
        }

        let parts: Vec<&str> = ip.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        // Basic IPv4 validation
        let addr_parts: Vec<&str> = parts[0].split('.').collect();
        if addr_parts.len() == 4 {
            for part in addr_parts {
                if let Ok(num) = part.parse::<u16>() {
                    if num > 255 {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Check prefix length
            if let Ok(prefix) = parts[1].parse::<u8>() {
                return prefix <= 32;
            }
        }

        // Basic IPv6 validation (simplified)
        if parts[0].contains(':') {
            if let Ok(prefix) = parts[1].parse::<u8>() {
                return prefix <= 128;
            }
        }

        false
    }

    fn is_valid_vlan_id(vlan_id: u16) -> bool {
        (1..=4094).contains(&vlan_id)
    }
}
