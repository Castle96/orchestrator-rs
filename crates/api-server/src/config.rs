use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cluster: ClusterConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: Option<usize>,
    pub keepalive: Option<u64>,
    pub client_timeout: Option<u64>,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_file: PathBuf,
    pub key_file: PathBuf,
    pub ca_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub acquire_timeout: Option<u64>,
    pub idle_timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub node_id: Option<String>,
    pub node_name: String,
    pub bind_address: String,
    pub bind_port: u16,
    pub advertise_address: Option<String>,
    pub join_addresses: Vec<String>,
    pub election_timeout: Option<u64>,
    pub heartbeat_interval: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_path: PathBuf,
    pub default_pool: String,
    pub pool_configs: Vec<PoolConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub name: String,
    pub storage_type: String,
    pub path: String,
    pub options: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub default_bridge: String,
    pub bridge_prefix: String,
    pub ip_range: String,
    pub dns_servers: Vec<String>,
    pub firewall_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: Option<String>,
    pub file: Option<PathBuf>,
    pub rotate: Option<bool>,
    pub max_files: Option<u32>,
    pub max_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub auth_enabled: bool,
    pub jwt_secret: Option<String>,
    pub jwt_expiry: Option<u64>,
    pub api_keys: Vec<String>,
    pub cors_origins: Vec<String>,
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: Some(num_cpus::get()),
                max_connections: Some(1000),
                keepalive: Some(30),
                client_timeout: Some(60),
                tls: None,
            },
            database: DatabaseConfig {
                url: "sqlite:///var/lib/arm-hypervisor/database.db".to_string(),
                max_connections: Some(10),
                min_connections: Some(1),
                acquire_timeout: Some(30),
                idle_timeout: Some(600),
            },
            cluster: ClusterConfig {
                node_id: None,
                node_name: gethostname::gethostname().to_string_lossy().to_string(),
                bind_address: "0.0.0.0".to_string(),
                bind_port: 7946,
                advertise_address: None,
                join_addresses: vec![],
                election_timeout: Some(5000),
                heartbeat_interval: Some(1000),
            },
            storage: StorageConfig {
                base_path: PathBuf::from("/var/lib/arm-hypervisor/storage"),
                default_pool: "default".to_string(),
                pool_configs: vec![PoolConfig {
                    name: "default".to_string(),
                    storage_type: "local".to_string(),
                    path: "/var/lib/arm-hypervisor/storage/default".to_string(),
                    options: std::collections::HashMap::new(),
                }],
            },
            network: NetworkConfig {
                default_bridge: "lxcbr0".to_string(),
                bridge_prefix: "hvbr".to_string(),
                ip_range: "192.168.100.0/24".to_string(),
                dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
                firewall_enabled: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: Some("json".to_string()),
                file: Some(PathBuf::from("/var/log/arm-hypervisor/hypervisor.log")),
                rotate: Some(true),
                max_files: Some(10),
                max_size: Some("100MB".to_string()),
            },
            security: SecurityConfig {
                auth_enabled: true,
                jwt_secret: None,
                jwt_expiry: Some(86400), // 24 hours
                api_keys: vec![],
                cors_origins: vec!["*".to_string()],
                rate_limit: Some(RateLimitConfig {
                    requests_per_minute: 60,
                    burst_size: 10,
                }),
            },
        }
    }
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = if path.ends_with(".toml") {
            toml::from_str(&content)?
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };
        Ok(config)
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Server config from env
        if let Ok(host) = std::env::var("SERVER_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("SERVER_PORT") {
            if let Ok(port) = port.parse() {
                config.server.port = port;
            }
        }

        // Database config from env
        if let Ok(url) = std::env::var("DATABASE_URL") {
            config.database.url = url;
        }

        // Cluster config from env
        if let Ok(node_name) = std::env::var("CLUSTER_NODE_NAME") {
            config.cluster.node_name = node_name;
        }
        if let Ok(bind_addr) = std::env::var("CLUSTER_BIND_ADDRESS") {
            config.cluster.bind_address = bind_addr;
        }
        if let Ok(bind_port) = std::env::var("CLUSTER_BIND_PORT") {
            if let Ok(port) = bind_port.parse() {
                config.cluster.bind_port = port;
            }
        }

        // Logging config from env
        if let Ok(level) = std::env::var("LOG_LEVEL") {
            config.logging.level = level;
        }

        // Security config from env
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.security.jwt_secret = Some(jwt_secret);
        }
        if let Ok(auth) = std::env::var("AUTH_ENABLED") {
            config.security.auth_enabled = auth.parse().unwrap_or(true);
        }

        config
    }

    pub fn merge_with_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_config = Self::from_file(path)?;

        // Merge configurations (file overrides defaults, env overrides file)
        self.server.host = file_config.server.host;
        self.server.port = file_config.server.port;
        self.server.workers = file_config.server.workers.or(self.server.workers);
        self.server.max_connections = file_config
            .server
            .max_connections
            .or(self.server.max_connections);
        self.server.keepalive = file_config.server.keepalive.or(self.server.keepalive);
        self.server.client_timeout = file_config
            .server
            .client_timeout
            .or(self.server.client_timeout);
        self.server.tls = file_config.server.tls.or(self.server.tls.clone());

        self.database.url = file_config.database.url;
        self.database.max_connections = file_config
            .database
            .max_connections
            .or(self.database.max_connections);
        self.database.min_connections = file_config
            .database
            .min_connections
            .or(self.database.min_connections);
        self.database.acquire_timeout = file_config
            .database
            .acquire_timeout
            .or(self.database.acquire_timeout);
        self.database.idle_timeout = file_config
            .database
            .idle_timeout
            .or(self.database.idle_timeout);

        self.cluster = file_config.cluster;

        self.storage = file_config.storage;
        self.network = file_config.network;

        self.logging.level = file_config.logging.level;
        self.logging.format = file_config.logging.format.or(self.logging.format.clone());
        self.logging.file = file_config.logging.file.or(self.logging.file.clone());
        self.logging.rotate = file_config.logging.rotate.or(self.logging.rotate);
        self.logging.max_files = file_config.logging.max_files.or(self.logging.max_files);
        self.logging.max_size = file_config
            .logging
            .max_size
            .or(self.logging.max_size.clone());

        self.security.auth_enabled = file_config.security.auth_enabled;
        self.security.jwt_secret = file_config
            .security
            .jwt_secret
            .or(self.security.jwt_secret.clone());
        self.security.jwt_expiry = file_config.security.jwt_expiry.or(self.security.jwt_expiry);
        self.security.api_keys = file_config.security.api_keys;
        self.security.cors_origins = file_config.security.cors_origins;
        self.security.rate_limit = file_config
            .security
            .rate_limit
            .or(self.security.rate_limit.clone());

        Ok(())
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate server config
        if self.server.host.is_empty() {
            errors.push("Server host cannot be empty".to_string());
        }
        if self.server.port == 0 {
            errors.push("Server port must be greater than 0".to_string());
        }

        // Validate database config
        if self.database.url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }

        // Validate cluster config
        if self.cluster.node_name.is_empty() {
            errors.push("Cluster node name cannot be empty".to_string());
        }
        if self.cluster.bind_address.is_empty() {
            errors.push("Cluster bind address cannot be empty".to_string());
        }

        // Validate storage config
        if self.storage.default_pool.is_empty() {
            errors.push("Default storage pool name cannot be empty".to_string());
        }
        if self.storage.pool_configs.is_empty() {
            errors.push("At least one storage pool configuration is required".to_string());
        }

        // Validate network config
        if self.network.default_bridge.is_empty() {
            errors.push("Default bridge name cannot be empty".to_string());
        }

        // Validate logging config
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.logging.level));
        }

        // Validate security config
        if self.security.auth_enabled {
            if let Some(ref secret) = self.security.jwt_secret {
                // Check for default/weak JWT secrets
                let weak_secrets = [
                    "your-super-secret-jwt-key-change-this-in-production",
                    "secret",
                    "changeme",
                    "password",
                    "12345678",
                ];
                if weak_secrets.contains(&secret.as_str()) {
                    errors.push("CRITICAL SECURITY ERROR: JWT secret is using a default or weak value. Change it immediately!".to_string());
                }
                // Enforce minimum length
                if secret.len() < 32 {
                    errors.push(
                        "JWT secret must be at least 32 characters long for security".to_string(),
                    );
                }
            } else {
                errors.push("JWT secret is required when authentication is enabled".to_string());
            }
        }

        // Warn about permissive CORS
        if self.security.cors_origins.contains(&"*".to_string()) {
            eprintln!("WARNING: CORS is configured to allow all origins (*). This should not be used in production.");
        }

        // Validate TLS configuration if present
        if let Some(ref tls) = self.server.tls {
            if !tls.cert_file.exists() {
                errors.push(format!(
                    "TLS certificate file not found: {:?}",
                    tls.cert_file
                ));
            }
            if !tls.key_file.exists() {
                errors.push(format!("TLS key file not found: {:?}", tls.key_file));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let mut config = AppConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);

        // Default config has auth enabled but no JWT secret, which should fail validation
        assert!(config.validate().is_err());

        // Adding a valid JWT secret should make it pass
        config.security.jwt_secret =
            Some("a-very-long-secure-jwt-secret-that-is-at-least-32-characters".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        // Add valid JWT secret for auth
        config.security.jwt_secret =
            Some("a-very-long-secure-jwt-secret-that-is-at-least-32-characters".to_string());

        // Test invalid server config
        config.server.host = "".to_string();
        assert!(config.validate().is_err());

        // Fix and test valid config
        config.server.host = "localhost".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_env_config_override() {
        std::env::set_var("SERVER_HOST", "127.0.0.1");
        std::env::set_var("SERVER_PORT", "9090");

        let config = AppConfig::from_env();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9090);

        std::env::remove_var("SERVER_HOST");
        std::env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_jwt_secret_validation() {
        let mut config = AppConfig::default();
        config.security.auth_enabled = true;

        // Test with no JWT secret
        config.security.jwt_secret = None;
        assert!(config.validate().is_err());

        // Test with weak JWT secret
        config.security.jwt_secret =
            Some("your-super-secret-jwt-key-change-this-in-production".to_string());
        let result = config.validate();
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("CRITICAL SECURITY ERROR")));
        }

        // Test with short JWT secret
        config.security.jwt_secret = Some("short".to_string());
        let result = config.validate();
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("at least 32 characters")));
        }

        // Test with valid JWT secret
        config.security.jwt_secret =
            Some("a-very-long-and-secure-jwt-secret-key-that-is-definitely-not-weak".to_string());
        assert!(config.validate().is_ok());
    }
}
