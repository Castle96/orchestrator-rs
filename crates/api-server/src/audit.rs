/// Audit logging module for tracking all system operations
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    // Container actions
    ContainerCreated,
    ContainerDeleted,
    ContainerStarted,
    ContainerStopped,
    ContainerUpdated,
    ContainerSnapshotCreated,
    ContainerSnapshotRestored,
    ContainerSnapshotDeleted,
    ContainerCloned,
    
    // User actions
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserLogin,
    UserLogout,
    
    // Cluster actions
    ClusterJoined,
    ClusterLeft,
    ClusterNodeAdded,
    ClusterNodeRemoved,
    
    // Storage actions
    StoragePoolCreated,
    StoragePoolDeleted,
    VolumeCreated,
    VolumeDeleted,
    
    // Network actions
    BridgeCreated,
    BridgeDeleted,
    NetworkInterfaceCreated,
    NetworkInterfaceDeleted,
    
    // System actions
    ConfigurationChanged,
    SystemStarted,
    SystemStopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user: Option<String>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub result: AuditResult,
    pub ip_address: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub details: Option<String>,
}

/// In-memory audit log storage (in production, use a persistent store)
pub struct AuditLogger {
    logs: Mutex<Vec<AuditLog>>,
    max_logs: usize,
}

/// Builder for creating audit log entries
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct AuditLogBuilder {
    user: Option<String>,
    action: Option<AuditAction>,
    resource_type: Option<String>,
    resource_id: Option<String>,
    result: Option<AuditResult>,
    ip_address: Option<String>,
    correlation_id: Option<Uuid>,
    details: Option<String>,
}

#[allow(dead_code)]
impl AuditLogBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn action(mut self, action: AuditAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn resource_type(mut self, resource_type: String) -> Self {
        self.resource_type = Some(resource_type);
        self
    }

    pub fn resource_id(mut self, resource_id: String) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn result(mut self, result: AuditResult) -> Self {
        self.result = Some(result);
        self
    }

    pub fn ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    pub fn correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn build(self) -> Result<AuditLog, &'static str> {
        Ok(AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user: self.user,
            action: self.action.ok_or("Action is required")?,
            resource_type: self.resource_type.ok_or("Resource type is required")?,
            resource_id: self.resource_id,
            result: self.result.ok_or("Result is required")?,
            ip_address: self.ip_address,
            correlation_id: self.correlation_id,
            details: self.details,
        })
    }
}

impl AuditLogger {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Mutex::new(Vec::new()),
            max_logs,
        }
    }

    /// Create a new audit log builder
    #[allow(dead_code)]
    pub fn builder() -> AuditLogBuilder {
        AuditLogBuilder::new()
    }

    /// Log an audit event using the builder pattern
    #[allow(dead_code)]
    pub fn log_entry(&self, log: AuditLog) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(log);

        // Keep only the most recent logs
        if logs.len() > self.max_logs {
            logs.remove(0);
        }
    }

    /// Get audit logs with optional filtering
    pub fn get_logs(
        &self,
        user: Option<String>,
        action: Option<AuditAction>,
        resource_type: Option<String>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let logs = self.logs.lock().unwrap();
        let mut filtered: Vec<AuditLog> = logs
            .iter()
            .filter(|log| {
                if let Some(ref u) = user {
                    if log.user.as_ref() != Some(u) {
                        return false;
                    }
                }
                if let Some(ref a) = action {
                    if std::mem::discriminant(&log.action) != std::mem::discriminant(a) {
                        return false;
                    }
                }
                if let Some(ref rt) = resource_type {
                    if &log.resource_type != rt {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by timestamp descending (most recent first)
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Limit results
        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Get the total number of logs
    pub fn count(&self) -> usize {
        self.logs.lock().unwrap().len()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000) // Keep last 10,000 logs by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger() {
        let logger = AuditLogger::new(100);

        // Log a successful action using builder
        let log = AuditLogger::builder()
            .user("admin".to_string())
            .action(AuditAction::ContainerCreated)
            .resource_type("container".to_string())
            .resource_id("test-container".to_string())
            .result(AuditResult::Success)
            .ip_address("192.168.1.100".to_string())
            .details("Created alpine container".to_string())
            .build()
            .unwrap();
        logger.log_entry(log);

        assert_eq!(logger.count(), 1);

        // Log a failed action
        let log = AuditLogger::builder()
            .user("user1".to_string())
            .action(AuditAction::ContainerDeleted)
            .resource_type("container".to_string())
            .resource_id("test-container".to_string())
            .result(AuditResult::Failure("Permission denied".to_string()))
            .ip_address("192.168.1.101".to_string())
            .build()
            .unwrap();
        logger.log_entry(log);

        assert_eq!(logger.count(), 2);

        // Get all logs
        let all_logs = logger.get_logs(None, None, None, None);
        assert_eq!(all_logs.len(), 2);

        // Filter by user
        let admin_logs = logger.get_logs(Some("admin".to_string()), None, None, None);
        assert_eq!(admin_logs.len(), 1);
        assert_eq!(admin_logs[0].user, Some("admin".to_string()));

        // Filter by resource type
        let container_logs = logger.get_logs(None, None, Some("container".to_string()), None);
        assert_eq!(container_logs.len(), 2);

        // Limit results
        let limited = logger.get_logs(None, None, None, Some(1));
        assert_eq!(limited.len(), 1);
    }

    #[test]
    fn test_max_logs() {
        let logger = AuditLogger::new(5);

        // Add 10 logs
        for i in 0..10 {
            let log = AuditLogger::builder()
                .user(format!("user{}", i))
                .action(AuditAction::UserLogin)
                .resource_type("user".to_string())
                .result(AuditResult::Success)
                .build()
                .unwrap();
            logger.log_entry(log);
        }

        // Should only keep the last 5
        assert_eq!(logger.count(), 5);
    }

    #[test]
    fn test_audit_log_builder() {
        let log = AuditLogger::builder()
            .user("admin".to_string())
            .action(AuditAction::ContainerCreated)
            .resource_type("container".to_string())
            .result(AuditResult::Success)
            .build();

        assert!(log.is_ok());
        let log = log.unwrap();
        assert_eq!(log.user, Some("admin".to_string()));
    }

    #[test]
    fn test_audit_log_builder_missing_required() {
        // Missing action
        let log = AuditLogger::builder()
            .user("admin".to_string())
            .resource_type("container".to_string())
            .result(AuditResult::Success)
            .build();

        assert!(log.is_err());
    }
}
