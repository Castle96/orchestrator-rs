use actix_web::{web, HttpResponse, Responder};
use tracing::{error, info};
use uuid::Uuid;

use ::network::{BridgeManager, NetworkError};
use ::storage::{LocalStorageManager, SharedStorageManager, StorageError};
use container_manager::{ContainerError, ContainerManager};
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
