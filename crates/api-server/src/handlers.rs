use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::collections::HashMap;
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

pub async fn health_check() -> impl Responder {
    info!("Health check requested");

    let mut status = HashMap::new();
    let mut overall_healthy = true;

    // Check container manager health
    match ContainerManager::list().await {
        Ok(_) => {
            status.insert("container_manager", json!({"status": "healthy"}));
        }
        Err(e) => {
            status.insert(
                "container_manager",
                json!({
                    "status": "unhealthy",
                    "error": e.to_string()
                }),
            );
            overall_healthy = false;
        }
    }

    // Check network manager health
    match BridgeManager::list().await {
        Ok(_) => {
            status.insert("network_manager", json!({"status": "healthy"}));
        }
        Err(e) => {
            status.insert(
                "network_manager",
                json!({
                    "status": "unhealthy",
                    "error": format!("{}", e)
                }),
            );
            overall_healthy = false;
        }
    }

    let response = json!({
        "status": if overall_healthy { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "services": status
    });

    if overall_healthy {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

pub async fn metrics() -> impl Responder {
    info!("Metrics requested");

    let mut metrics = HashMap::new();

    // System metrics
    if let Ok(load_avg) = sys_info::loadavg() {
        metrics.insert("system_load_1min", json!(load_avg.one));
        metrics.insert("system_load_5min", json!(load_avg.five));
        metrics.insert("system_load_15min", json!(load_avg.fifteen));
    }

    if let Ok(mem_info) = sys_info::mem_info() {
        metrics.insert("memory_total_kb", json!(mem_info.total));
        metrics.insert("memory_free_kb", json!(mem_info.free));
        metrics.insert("memory_available_kb", json!(mem_info.avail));
    }

    if let Ok(disk_info) = sys_info::disk_info() {
        metrics.insert("disk_total_kb", json!(disk_info.total));
        metrics.insert("disk_free_kb", json!(disk_info.free));
    }

    // Container metrics
    match ContainerManager::list().await {
        Ok(containers) => {
            metrics.insert("containers_total", json!(containers.len()));

            let mut running_count = 0;
            let mut stopped_count = 0;
            let mut error_count = 0;

            for container_name in containers {
                match ContainerManager::status(&container_name).await {
                    Ok(status) => match status {
                        ContainerStatus::Running => running_count += 1,
                        ContainerStatus::Stopped => stopped_count += 1,
                        ContainerStatus::Error => error_count += 1,
                        _ => {}
                    },
                    Err(_) => error_count += 1,
                }
            }

            metrics.insert("containers_running", json!(running_count));
            metrics.insert("containers_stopped", json!(stopped_count));
            metrics.insert("containers_error", json!(error_count));
        }
        Err(_) => {
            metrics.insert("containers_total", json!(-1));
        }
    }

    // Network metrics
    match BridgeManager::list().await {
        Ok(bridges) => {
            metrics.insert("bridges_total", json!(bridges.len()));
        }
        Err(_) => {
            metrics.insert("bridges_total", json!(-1));
        }
    }

    let response = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metrics": metrics
    });

    HttpResponse::Ok().json(response)
}
