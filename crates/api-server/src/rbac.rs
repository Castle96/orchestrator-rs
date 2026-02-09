/// Role-Based Access Control (RBAC) module
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    // Container permissions
    ContainerCreate,
    ContainerRead,
    ContainerUpdate,
    ContainerDelete,
    ContainerStart,
    ContainerStop,
    ContainerSnapshot,

    // Cluster permissions
    ClusterRead,
    ClusterWrite,
    ClusterJoin,
    ClusterLeave,

    // Storage permissions
    StorageRead,
    StorageWrite,
    StorageDelete,

    // Network permissions
    NetworkRead,
    NetworkWrite,
    NetworkDelete,

    // System permissions
    SystemRead,
    SystemWrite,
    SystemAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Operator,
    Viewer,
    Custom(String),
}

impl Role {
    /// Get the permissions for a role
    #[allow(dead_code)]
    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            Role::Admin => vec![
                Permission::ContainerCreate,
                Permission::ContainerRead,
                Permission::ContainerUpdate,
                Permission::ContainerDelete,
                Permission::ContainerStart,
                Permission::ContainerStop,
                Permission::ContainerSnapshot,
                Permission::ClusterRead,
                Permission::ClusterWrite,
                Permission::ClusterJoin,
                Permission::ClusterLeave,
                Permission::StorageRead,
                Permission::StorageWrite,
                Permission::StorageDelete,
                Permission::NetworkRead,
                Permission::NetworkWrite,
                Permission::NetworkDelete,
                Permission::SystemRead,
                Permission::SystemWrite,
                Permission::SystemAdmin,
            ],
            Role::Operator => vec![
                Permission::ContainerRead,
                Permission::ContainerStart,
                Permission::ContainerStop,
                Permission::ContainerSnapshot,
                Permission::ClusterRead,
                Permission::StorageRead,
                Permission::NetworkRead,
                Permission::SystemRead,
            ],
            Role::Viewer => vec![
                Permission::ContainerRead,
                Permission::ClusterRead,
                Permission::StorageRead,
                Permission::NetworkRead,
                Permission::SystemRead,
            ],
            Role::Custom(_) => vec![], // Custom roles need explicit permission assignment
        }
    }

    /// Check if this role has a specific permission
    #[allow(dead_code)]
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions().contains(permission)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: Role,
    pub custom_permissions: Vec<Permission>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    /// Check if the user has a specific permission
    #[allow(dead_code)]
    pub fn has_permission(&self, permission: &Permission) -> bool {
        if !self.enabled {
            return false;
        }

        // Check role permissions
        if self.role.has_permission(permission) {
            return true;
        }

        // Check custom permissions
        self.custom_permissions.contains(permission)
    }

    /// Check if the user has any of the specified permissions
    #[allow(dead_code)]
    pub fn has_any_permission(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.has_permission(p))
    }

    /// Check if the user has all of the specified permissions
    #[allow(dead_code)]
    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.has_permission(p))
    }
}

/// In-memory user store (in production, use a database)
pub struct UserStore {
    users: HashMap<String, User>,
}

impl UserStore {
    pub fn new() -> Self {
        let mut store = Self {
            users: HashMap::new(),
        };

        // Create default admin user
        let admin = User {
            id: Uuid::new_v4(),
            username: "admin".to_string(),
            email: Some("admin@example.com".to_string()),
            role: Role::Admin,
            custom_permissions: vec![],
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        store.users.insert("admin".to_string(), admin);

        store
    }

    pub fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.username.clone(), user);
    }

    pub fn update_user(&mut self, username: &str, user: User) -> Result<(), &'static str> {
        if self.users.contains_key(username) {
            self.users.insert(username.to_string(), user);
            Ok(())
        } else {
            Err("User not found")
        }
    }

    pub fn delete_user(&mut self, username: &str) -> Result<(), &'static str> {
        if username == "admin" {
            return Err("Cannot delete admin user");
        }

        if self.users.remove(username).is_some() {
            Ok(())
        } else {
            Err("User not found")
        }
    }

    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}

impl Default for UserStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        let admin_role = Role::Admin;
        assert!(admin_role.has_permission(&Permission::ContainerCreate));
        assert!(admin_role.has_permission(&Permission::SystemAdmin));
        assert!(admin_role.has_permission(&Permission::ClusterWrite));
    }

    #[test]
    fn test_viewer_has_limited_permissions() {
        let viewer_role = Role::Viewer;
        assert!(viewer_role.has_permission(&Permission::ContainerRead));
        assert!(!viewer_role.has_permission(&Permission::ContainerCreate));
        assert!(!viewer_role.has_permission(&Permission::SystemAdmin));
    }

    #[test]
    fn test_operator_permissions() {
        let operator_role = Role::Operator;
        assert!(operator_role.has_permission(&Permission::ContainerRead));
        assert!(operator_role.has_permission(&Permission::ContainerStart));
        assert!(!operator_role.has_permission(&Permission::ContainerCreate));
        assert!(!operator_role.has_permission(&Permission::SystemAdmin));
    }

    #[test]
    fn test_user_has_permission() {
        let user = User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: None,
            role: Role::Viewer,
            custom_permissions: vec![Permission::ContainerStart],
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(user.has_permission(&Permission::ContainerRead));
        assert!(user.has_permission(&Permission::ContainerStart)); // from custom permissions
        assert!(!user.has_permission(&Permission::ContainerCreate));
    }

    #[test]
    fn test_disabled_user_has_no_permissions() {
        let user = User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: None,
            role: Role::Admin,
            custom_permissions: vec![],
            enabled: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(!user.has_permission(&Permission::ContainerRead));
        assert!(!user.has_permission(&Permission::SystemAdmin));
    }

    #[test]
    fn test_user_store() {
        let mut store = UserStore::new();

        // Default admin should exist
        assert!(store.get_user("admin").is_some());

        // Add a new user
        let user = User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: None,
            role: Role::Viewer,
            custom_permissions: vec![],
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        store.add_user(user);

        assert!(store.get_user("testuser").is_some());
        assert_eq!(store.list_users().len(), 2);

        // Cannot delete admin
        assert!(store.delete_user("admin").is_err());

        // Can delete other users
        assert!(store.delete_user("testuser").is_ok());
        assert_eq!(store.list_users().len(), 1);
    }
}
