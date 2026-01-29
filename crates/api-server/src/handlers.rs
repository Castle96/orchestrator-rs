use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;

use ::network::{BridgeManager, NetworkError};
use ::storage::{LocalStorageManager, SharedStorageManager, StorageError};
use container_manager::{ContainerError, ContainerManager, SnapshotManager};
use models::*;

pub async fn list_containers() -> impl Responder {
    info!("Listing containers");

    match ContainerManager::list().await {
        Ok(container_names) => {
            // In production, you'd fetch full container details
            let containers: Vec<Container> = container_names
                .into_iter()
                .map(|name| {
                    // Simplified - in production, get from database
                    Container {
                        id: Uuid::new_v4(),
                        name: name.clone(),
                        status: ContainerStatus::Stopped,
                        template: "unknown".to_string(),
                        node_id: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        config: ContainerConfig {
                            cpu_limit: None,
                            memory_limit: None,
                            disk_limit: None,
                            network_interfaces: vec![],
                            rootfs_path: format!("/var/lib/lxc/{}/rootfs", name),
                            environment: vec![],
                        },
                    }
                })
                .collect();

            HttpResponse::Ok().json(ContainerListResponse { containers })
        }
        Err(e) => {
            error!("Failed to list containers: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn create_container(req: web::Json<CreateContainerRequest>) -> impl Responder {
    info!("Creating container: {}", req.name);

    match ContainerManager::create(req.into_inner()).await {
        Ok(container) => HttpResponse::Created().json(ContainerResponse { container }),
        Err(ContainerError::AlreadyExists(name)) => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": format!("Container already exists: {}", name)
            }))
        }
        Err(e) => {
            error!("Failed to create container: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn get_container(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    info!("Getting container: {}", name);

    match ContainerManager::get(&name).await {
        Ok(container) => HttpResponse::Ok().json(ContainerResponse { container }),
        Err(ContainerError::NotFound(name)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Container not found: {}", name)
        })),
        Err(e) => {
            error!("Failed to get container: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn start_container(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    info!("Starting container: {}", name);

    match ContainerManager::start(&name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Container {} started", name)
        })),
        Err(ContainerError::NotFound(name)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Container not found: {}", name)
        })),
        Err(e) => {
            error!("Failed to start container: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn stop_container(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    info!("Stopping container: {}", name);

    match ContainerManager::stop(&name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Container {} stopped", name)
        })),
        Err(ContainerError::NotFound(name)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Container not found: {}", name)
        })),
        Err(e) => {
            error!("Failed to stop container: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn delete_container(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    info!("Deleting container: {}", name);

    match ContainerManager::delete(&name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Container {} deleted", name)
        })),
        Err(ContainerError::NotFound(name)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Container not found: {}", name)
        })),
        Err(e) => {
            error!("Failed to delete container: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn list_nodes() -> impl Responder {
    info!("Listing cluster nodes");

    // In production, get from cluster manager
    HttpResponse::Ok().json(NodeListResponse { nodes: vec![] })
}

pub async fn join_cluster(req: web::Json<JoinClusterRequest>) -> impl Responder {
    info!("Joining cluster: {}", req.cluster_name);

    // In production, implement cluster join logic
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Cluster join initiated"
    }))
}

pub async fn cluster_status() -> impl Responder {
    info!("Getting cluster status");

    // In production, get from cluster manager
    HttpResponse::Ok().json(serde_json::json!({
        "cluster": {
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "default",
            "node_count": 0
        }
    }))
}

pub async fn list_storage_pools() -> impl Responder {
    info!("Listing storage pools");

    // In production, get from storage manager
    HttpResponse::Ok().json(StoragePoolListResponse { pools: vec![] })
}

pub async fn create_storage_pool(req: web::Json<CreateStoragePoolRequest>) -> impl Responder {
    info!("Creating storage pool: {}", req.name);

    let result: Result<StoragePool, StorageError> = match req.storage_type {
        StorageType::Local => LocalStorageManager::create_pool(&req.name, &req.path).await,
        StorageType::Nfs => {
            // Parse NFS path (server:path)
            let parts: Vec<&str> = req.path.split(':').collect();
            if parts.len() != 2 {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid NFS path format. Expected server:path"
                }));
            }
            SharedStorageManager::create_nfs_pool(&req.name, parts[0], parts[1]).await
        }
        StorageType::Cifs => {
            // Parse CIFS path (//server/share)
            if !req.path.starts_with("//") {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid CIFS path format. Expected //server/share"
                }));
            }
            let path = req.path.trim_start_matches("//");
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() != 2 {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid CIFS path format. Expected //server/share"
                }));
            }
            SharedStorageManager::create_cifs_pool(&req.name, parts[0], parts[1], None).await
        }
    };

    match result {
        Ok(pool) => HttpResponse::Created().json(pool),
        Err(e) => {
            error!("Failed to create storage pool: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn list_network_interfaces() -> impl Responder {
    info!("Listing network interfaces");

    // In production, get from network manager
    HttpResponse::Ok().json(NetworkListResponse { interfaces: vec![] })
}

pub async fn list_bridges() -> impl Responder {
    info!("Listing bridges");

    match BridgeManager::list().await {
        Ok(bridge_names) => {
            // In production, get full bridge details
            HttpResponse::Ok().json(serde_json::json!({
                "bridges": bridge_names
            }))
        }
        Err(e) => {
            error!("Failed to list bridges: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e)
            }))
        }
    }
}

pub async fn create_bridge(req: web::Json<CreateBridgeRequest>) -> impl Responder {
    info!("Creating bridge: {}", req.name);

    match BridgeManager::create(req.into_inner()).await {
        Ok(bridge) => HttpResponse::Created().json(bridge),
        Err(NetworkError::BridgeExists(name)) => HttpResponse::Conflict().json(serde_json::json!({
            "error": format!("Bridge already exists: {}", name)
        })),
        Err(e) => {
            error!("Failed to create bridge: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e)
            }))
        }
    }
}

// ============================================================================
// Container Snapshot Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotRequest {
    pub name: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreSnapshotRequest {
    pub snapshot_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CloneFromSnapshotRequest {
    pub snapshot_name: String,
    pub new_container_name: String,
}

/// List all snapshots for a container
pub async fn list_snapshots(path: web::Path<String>) -> impl Responder {
    let container_name = path.into_inner();
    info!("Listing snapshots for container: {}", container_name);

    match SnapshotManager::list(&container_name).await {
        Ok(snapshots) => HttpResponse::Ok().json(serde_json::json!({
            "container": container_name,
            "snapshots": snapshots
        })),
        Err(ContainerError::NotFound(name)) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Container not found: {}", name)
            }))
        }
        Err(e) => {
            error!("Failed to list snapshots: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// Create a snapshot of a container
pub async fn create_snapshot(
    path: web::Path<String>,
    req: web::Json<CreateSnapshotRequest>,
) -> impl Responder {
    let container_name = path.into_inner();
    info!("Creating snapshot for container: {}", container_name);

    match SnapshotManager::create(&container_name, req.name.clone(), req.comment.clone()).await {
        Ok(snapshot) => HttpResponse::Created().json(serde_json::json!({
            "message": "Snapshot created successfully",
            "snapshot": snapshot
        })),
        Err(ContainerError::NotFound(name)) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Container not found: {}", name)
            }))
        }
        Err(e) => {
            error!("Failed to create snapshot: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// Restore a container from a snapshot
pub async fn restore_snapshot(
    path: web::Path<String>,
    req: web::Json<RestoreSnapshotRequest>,
) -> impl Responder {
    let container_name = path.into_inner();
    info!(
        "Restoring container '{}' from snapshot '{}'",
        container_name, req.snapshot_name
    );

    match SnapshotManager::restore(&container_name, &req.snapshot_name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!(
                "Container '{}' restored from snapshot '{}'",
                container_name, req.snapshot_name
            )
        })),
        Err(ContainerError::NotFound(name)) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Container not found: {}", name)
            }))
        }
        Err(e) => {
            error!("Failed to restore snapshot: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// Delete a snapshot
pub async fn delete_snapshot(path: web::Path<(String, String)>) -> impl Responder {
    let (container_name, snapshot_name) = path.into_inner();
    info!(
        "Deleting snapshot '{}' for container '{}'",
        snapshot_name, container_name
    );

    match SnapshotManager::delete(&container_name, &snapshot_name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Snapshot '{}' deleted", snapshot_name)
        })),
        Err(ContainerError::NotFound(name)) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Container not found: {}", name)
            }))
        }
        Err(e) => {
            error!("Failed to delete snapshot: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// Clone a container from a snapshot
pub async fn clone_from_snapshot(
    path: web::Path<String>,
    req: web::Json<CloneFromSnapshotRequest>,
) -> impl Responder {
    let container_name = path.into_inner();
    info!(
        "Cloning container '{}' from snapshot '{}' to '{}'",
        container_name, req.snapshot_name, req.new_container_name
    );

    match SnapshotManager::clone(&container_name, &req.snapshot_name, &req.new_container_name)
        .await
    {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "message": format!(
                "Container '{}' cloned from snapshot '{}' to '{}'",
                container_name, req.snapshot_name, req.new_container_name
            )
        })),
        Err(ContainerError::NotFound(name)) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Container not found: {}", name)
            }))
        }
        Err(ContainerError::AlreadyExists(name)) => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": format!("Container already exists: {}", name)
            }))
        }
        Err(e) => {
            error!("Failed to clone from snapshot: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

// ============================================================================
// User Management Handlers (RBAC)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub role: crate::rbac::Role,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub role: Option<crate::rbac::Role>,
    pub enabled: Option<bool>,
}

/// List all users
pub async fn list_users(
    user_store: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::rbac::UserStore>>>,
) -> impl Responder {
    info!("Listing users");

    let store = user_store.lock().unwrap();
    let users = store.list_users();

    HttpResponse::Ok().json(serde_json::json!({
        "users": users
    }))
}

/// Get a specific user
pub async fn get_user(
    path: web::Path<String>,
    user_store: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::rbac::UserStore>>>,
) -> impl Responder {
    let username = path.into_inner();
    info!("Getting user: {}", username);

    let store = user_store.lock().unwrap();
    match store.get_user(&username) {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("User not found: {}", username)
        })),
    }
}

/// Create a new user
pub async fn create_user(
    req: web::Json<CreateUserRequest>,
    user_store: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::rbac::UserStore>>>,
) -> impl Responder {
    info!("Creating user: {}", req.username);

    let user = crate::rbac::User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        email: req.email.clone(),
        role: req.role.clone(),
        custom_permissions: vec![],
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut store = user_store.lock().unwrap();
    if store.get_user(&req.username).is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": format!("User already exists: {}", req.username)
        }));
    }

    store.add_user(user.clone());

    HttpResponse::Created().json(serde_json::json!({
        "message": "User created successfully",
        "user": user
    }))
}

/// Update a user
pub async fn update_user(
    path: web::Path<String>,
    req: web::Json<UpdateUserRequest>,
    user_store: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::rbac::UserStore>>>,
) -> impl Responder {
    let username = path.into_inner();
    info!("Updating user: {}", username);

    let mut store = user_store.lock().unwrap();
    let mut user = match store.get_user(&username) {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("User not found: {}", username)
            }))
        }
    };

    if let Some(email) = &req.email {
        user.email = Some(email.clone());
    }
    if let Some(role) = &req.role {
        user.role = role.clone();
    }
    if let Some(enabled) = req.enabled {
        user.enabled = enabled;
    }
    user.updated_at = chrono::Utc::now();

    match store.update_user(&username, user.clone()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "User updated successfully",
            "user": user
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// Delete a user
pub async fn delete_user_handler(
    path: web::Path<String>,
    user_store: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::rbac::UserStore>>>,
) -> impl Responder {
    let username = path.into_inner();
    info!("Deleting user: {}", username);

    let mut store = user_store.lock().unwrap();
    match store.delete_user(&username) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("User '{}' deleted successfully", username)
        })),
        Err(e) => {
            if e == "User not found" {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

// ============================================================================
// Audit Log Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub user: Option<String>,
    pub resource_type: Option<String>,
    pub limit: Option<usize>,
}

/// Get audit logs
pub async fn get_audit_logs(
    query: web::Query<AuditLogQuery>,
    audit_logger: actix_web::web::Data<std::sync::Arc<crate::audit::AuditLogger>>,
) -> impl Responder {
    info!("Getting audit logs");

    let logs = audit_logger.get_logs(
        query.user.clone(),
        None,
        query.resource_type.clone(),
        query.limit,
    );

    HttpResponse::Ok().json(serde_json::json!({
        "total": audit_logger.count(),
        "logs": logs
    }))
}
