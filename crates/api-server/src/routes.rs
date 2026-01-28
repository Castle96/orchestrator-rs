use actix_web::web;

use crate::{handlers, observability};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/containers", web::get().to(handlers::list_containers))
            .route("/containers", web::post().to(handlers::create_container))
            .route("/containers/{id}", web::get().to(handlers::get_container))
            .route(
                "/containers/{id}/start",
                web::post().to(handlers::start_container),
            )
            .route(
                "/containers/{id}/stop",
                web::post().to(handlers::stop_container),
            )
            .route(
                "/containers/{id}",
                web::delete().to(handlers::delete_container),
            )
            .route("/cluster/nodes", web::get().to(handlers::list_nodes))
            .route("/cluster/join", web::post().to(handlers::join_cluster))
            .route("/cluster/status", web::get().to(handlers::cluster_status))
            .route("/storage", web::get().to(handlers::list_storage_pools))
            .route("/storage", web::post().to(handlers::create_storage_pool))
            .route("/network", web::get().to(handlers::list_network_interfaces))
            .route("/network/bridges", web::get().to(handlers::list_bridges))
            .route("/network/bridges", web::post().to(handlers::create_bridge)),
    );

    // Add health and metrics endpoints (outside API versioning)
    cfg.route("/health", web::get().to(observability::health_check))
        .route("/ready", web::get().to(observability::readiness_check))
        .route("/metrics", web::get().to(observability::metrics_prometheus))
        .route("/metrics/json", web::get().to(observability::metrics_json));
}
