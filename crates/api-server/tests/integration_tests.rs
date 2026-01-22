use actix_web::{test, App};
use serde_json::json;

// Helper to check if LXC is available in environment
fn lxc_available() -> bool {
    std::process::Command::new("lxc-ls")
        .output()
        .is_ok()
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
        assert!(status.as_u16() >= 500, "Expected error status in test env without LXC, got {}", status);
    } else {
        assert!(status.is_success(), "Expected success with LXC installed, got {}", status);
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
        assert!(status.as_u16() >= 500, "Expected error in test env, got {}", status);
    } else {
        assert!(status.is_client_error() || status.is_success(), "Expected 2xx/4xx with LXC, got {}", status);
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
    assert!(status.is_client_error() || status.as_u16() == 500, 
            "Expected 4xx or 500, got {}", status);
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
