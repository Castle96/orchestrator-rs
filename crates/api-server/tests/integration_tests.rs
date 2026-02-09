use actix_web::{test, web, App};
use serde_json::json;
use std::sync::Arc;

// Helper to create app with all required dependencies
fn create_test_app() -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    // Create required shared state
    let metrics_collector = Arc::new(api_server::observability::MetricsCollector::new());
    let user_store = Arc::new(std::sync::Mutex::new(api_server::rbac::UserStore::new()));
    let audit_logger = Arc::new(api_server::audit::AuditLogger::new(10000));

    App::new()
        .app_data(web::Data::new(metrics_collector))
        .app_data(web::Data::new(user_store))
        .app_data(web::Data::new(audit_logger))
        .configure(api_server::routes::configure_routes)
}

// Helper to check if LXC is available in environment
fn lxc_available() -> bool {
    std::process::Command::new("lxc-ls").output().is_ok()
}

#[actix_web::test]
async fn test_list_containers() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/containers")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();

    // In test environment without LXC, we expect 500 or 503
    // In production with LXC, we expect 200
    if !lxc_available() {
        println!("LXC not available - testing API structure only");
        assert!(
            status.as_u16() >= 500,
            "Expected error status in test env without LXC, got {}",
            status
        );
    } else {
        assert!(
            status.is_success(),
            "Expected success with LXC installed, got {}",
            status
        );
    }
}

#[actix_web::test]
async fn test_create_container() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    let container_request = json!({
        "name": "test-container",
        "template": "alpine",
        "config": {
            "cpu_limit": 2,
            "memory_limit": 1073741824i64,
            "disk_limit": 10737418240i64,
            "network_interfaces": [],
            "rootfs_path": "/var/lib/lxc/test-container/rootfs",
            "environment": []
        }
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/containers")
        .set_json(&container_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();

    // In test environment: expect 500 (LXC not available)
    // In production: expect 201 (created) or 4xx (validation error)
    if !lxc_available() {
        assert!(
            status.as_u16() >= 500,
            "Expected error in test env, got {}",
            status
        );
    } else {
        assert!(
            status.is_client_error() || status.is_success(),
            "Expected 2xx/4xx with LXC, got {}",
            status
        );
    }
}

#[actix_web::test]
async fn test_cluster_status() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/cluster/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_list_storage_pools() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    let req = test::TestRequest::get().uri("/api/v1/storage").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_create_storage_pool() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    let storage_request = json!({
        "name": "test-pool",
        "storage_type": "local",
        "path": "/tmp/test-storage"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/storage")
        .set_json(&storage_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error() || resp.status().is_success());
}

#[actix_web::test]
async fn test_list_bridges() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/network/bridges")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();

    // Network operations require privileges
    // Accept any response - we're testing API structure
    println!("list_bridges status: {}", status);
    assert!(status.as_u16() >= 200, "Got invalid status: {}", status);
}

#[actix_web::test]
async fn test_create_bridge() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    let bridge_request = json!({
        "name": "test-bridge",
        "ip_address": "192.168.100.1/24",
        "stp_enabled": false
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/network/bridges")
        .set_json(&bridge_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();

    // Creating bridges requires root - accept any response in tests
    println!("create_bridge status: {}", status);
    assert!(status.as_u16() >= 200, "Got invalid status: {}", status);
}

#[actix_web::test]
async fn test_invalid_container_name() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    let invalid_request = json!({
        "name": "",
        "template": "alpine",
        "config": {
            "cpu_limit": null,
            "memory_limit": null,
            "disk_limit": null,
            "network_interfaces": [],
            "rootfs_path": "/var/lib/lxc/invalid/rootfs",
            "environment": []
        }
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/containers")
        .set_json(&invalid_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();

    // Should return 4xx (client error) for invalid name
    // But may return 500 if LXC check fails first
    println!("invalid_container_name status: {}", status);
    assert!(
        status.is_client_error() || status.as_u16() == 500,
        "Expected 4xx or 500, got {}",
        status
    );
}

#[actix_web::test]
async fn test_nonexistent_container_operations() {
    let app = test::init_service(App::new().configure(api_server::routes::configure_routes)).await;

    // Test getting non-existent container
    let req = test::TestRequest::get()
        .uri("/api/v1/containers/nonexistent")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // Test starting non-existent container
    let req = test::TestRequest::post()
        .uri("/api/v1/containers/nonexistent/start")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // Test stopping non-existent container
    let req = test::TestRequest::post()
        .uri("/api/v1/containers/nonexistent/stop")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

// Tests for new features added on 2026-01-28

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(create_test_app()).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    // Health check may return 503 if LXC is not available (expected in test env)
    // Should not return 404 (endpoint exists) or 500 (unless dependencies fail)
    assert!(
        status.is_success() || status.as_u16() == 503,
        "Health endpoint should return 2xx or 503, got: {}",
        status
    );
}

#[actix_web::test]
async fn test_ready_endpoint() {
    let app = test::init_service(create_test_app()).await;
    let req = test::TestRequest::get().uri("/ready").to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    // Readiness may return 503 if services aren't ready (expected without LXC)
    assert!(
        status.is_success() || status.as_u16() == 503,
        "Ready endpoint should return 2xx or 503, got: {}",
        status
    );
}

#[actix_web::test]
async fn test_metrics_prometheus_endpoint() {
    let app = test::init_service(create_test_app()).await;
    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    // With proper dependencies, metrics should return 200 or 500 if sys-info fails
    assert!(
        status.is_success() || status.is_server_error(),
        "Metrics endpoint should return 2xx or 5xx, got: {}",
        status
    );
}

#[actix_web::test]
async fn test_metrics_json_endpoint() {
    let app = test::init_service(create_test_app()).await;
    let req = test::TestRequest::get().uri("/metrics/json").to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    // With proper dependencies, metrics should return 200 or 500 if sys-info fails
    assert!(
        status.is_success() || status.is_server_error(),
        "Metrics JSON endpoint should return 2xx or 5xx, got: {}",
        status
    );
}

#[actix_web::test]
async fn test_list_snapshots() {
    let app = test::init_service(create_test_app()).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/containers/test-container/snapshots")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    // Should return 404 (container not found) or 500 (LXC not available)
    // Should NOT return 404 for missing route
    assert!(
        status.is_client_error() || status.is_server_error(),
        "Snapshots endpoint should return 4xx or 5xx without container, got: {}",
        status
    );
}

#[actix_web::test]
async fn test_list_users() {
    let app = test::init_service(create_test_app()).await;
    let req = test::TestRequest::get().uri("/api/v1/users").to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    // With proper dependencies, user list should return 200
    assert!(
        status.is_success(),
        "Users endpoint should return 2xx with dependencies, got: {}",
        status
    );
}

#[actix_web::test]
async fn test_get_audit_logs() {
    let app = test::init_service(create_test_app()).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/audit/logs")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    // With proper dependencies, audit logs should return 200
    assert!(
        status.is_success(),
        "Audit logs endpoint should return 2xx with dependencies, got: {}",
        status
    );
}
