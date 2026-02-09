/// Observability module providing enhanced monitoring and metrics
use actix_web::{HttpResponse, Responder};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::info;

use container_manager::ContainerManager;
use models::ContainerStatus;
use network::BridgeManager;

/// Global metrics collector
pub struct MetricsCollector {
    /// Total HTTP requests received
    pub http_requests_total: AtomicU64,
    /// Total HTTP errors
    pub http_errors_total: AtomicU64,
    /// Server start time
    pub start_time: SystemTime,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            http_requests_total: AtomicU64::new(0),
            http_errors_total: AtomicU64::new(0),
            start_time: SystemTime::now(),
        }
    }

    pub fn record_request(&self) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.http_errors_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().unwrap_or_default().as_secs()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced health check endpoint
pub async fn health_check() -> impl Responder {
    info!("Health check requested");

    let mut status = HashMap::new();
    let mut overall_healthy = true;
    let skip_system_checks = std::env::var("SKIP_SYSTEM_CHECKS")
        .map(|v| matches!(v.as_str(), "1" | "true" | "True" | "TRUE"))
        .unwrap_or(false);

    // Check container manager health
    if skip_system_checks {
        status.insert(
            "container_manager",
            json!({"status": "healthy", "note": "skipped system checks in dev mode"}),
        );
    } else {
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
    }

    // Check network manager health
    if skip_system_checks {
        status.insert(
            "network_manager",
            json!({"status": "healthy", "note": "skipped system checks in dev mode"}),
        );
    } else {
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

/// Readiness check endpoint (for k8s-style readiness probes)
/// Returns 200 if the service is ready to accept traffic
pub async fn readiness_check() -> impl Responder {
    info!("Readiness check requested");

    let skip_system_checks = std::env::var("SKIP_SYSTEM_CHECKS")
        .map(|v| matches!(v.as_str(), "1" | "true" | "True" | "TRUE"))
        .unwrap_or(false);

    // Check if critical services are operational
    let container_ready = if skip_system_checks {
        true
    } else {
        ContainerManager::list().await.is_ok()
    };
    let network_ready = if skip_system_checks {
        true
    } else {
        BridgeManager::list().await.is_ok()
    };

    if container_ready && network_ready {
        HttpResponse::Ok().json(json!({
            "status": "ready",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(json!({
            "status": "not_ready",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "container_manager": container_ready,
            "network_manager": network_ready
        }))
    }
}

/// Enhanced metrics endpoint with JSON format
pub async fn metrics_json(
    metrics_collector: actix_web::web::Data<Arc<MetricsCollector>>,
) -> impl Responder {
    info!("Metrics (JSON) requested");

    let mut metrics = HashMap::new();

    // Application metrics
    metrics.insert(
        "http_requests_total",
        json!(metrics_collector
            .http_requests_total
            .load(Ordering::Relaxed)),
    );
    metrics.insert(
        "http_errors_total",
        json!(metrics_collector.http_errors_total.load(Ordering::Relaxed)),
    );
    metrics.insert(
        "uptime_seconds",
        json!(metrics_collector.get_uptime_seconds()),
    );

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
        let used = mem_info.total.saturating_sub(mem_info.avail);
        let usage_percent = if mem_info.total > 0 {
            (used as f64 / mem_info.total as f64) * 100.0
        } else {
            0.0
        };
        metrics.insert("memory_usage_percent", json!(usage_percent));
    }

    if let Ok(disk_info) = sys_info::disk_info() {
        metrics.insert("disk_total_kb", json!(disk_info.total));
        metrics.insert("disk_free_kb", json!(disk_info.free));
        let used = disk_info.total.saturating_sub(disk_info.free);
        let usage_percent = if disk_info.total > 0 {
            (used as f64 / disk_info.total as f64) * 100.0
        } else {
            0.0
        };
        metrics.insert("disk_usage_percent", json!(usage_percent));
    }

    // CPU count
    metrics.insert("cpu_count", json!(num_cpus::get()));

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

/// Prometheus-compatible metrics endpoint
/// Exports metrics in Prometheus text format
pub async fn metrics_prometheus(
    metrics_collector: actix_web::web::Data<Arc<MetricsCollector>>,
) -> impl Responder {
    info!("Metrics (Prometheus) requested");

    let mut output = String::new();

    // Helper to add a metric
    let add_metric =
        |output: &mut String, name: &str, help: &str, metric_type: &str, value: String| {
            output.push_str(&format!("# HELP {} {}\n", name, help));
            output.push_str(&format!("# TYPE {} {}\n", name, metric_type));
            output.push_str(&format!("{} {}\n", name, value));
        };

    // Application metrics
    add_metric(
        &mut output,
        "arm_hypervisor_http_requests_total",
        "Total HTTP requests received",
        "counter",
        metrics_collector
            .http_requests_total
            .load(Ordering::Relaxed)
            .to_string(),
    );

    add_metric(
        &mut output,
        "arm_hypervisor_http_errors_total",
        "Total HTTP errors",
        "counter",
        metrics_collector
            .http_errors_total
            .load(Ordering::Relaxed)
            .to_string(),
    );

    add_metric(
        &mut output,
        "arm_hypervisor_uptime_seconds",
        "Server uptime in seconds",
        "gauge",
        metrics_collector.get_uptime_seconds().to_string(),
    );

    // System metrics
    if let Ok(load_avg) = sys_info::loadavg() {
        add_metric(
            &mut output,
            "arm_hypervisor_system_load_1min",
            "System load average (1 minute)",
            "gauge",
            load_avg.one.to_string(),
        );
        add_metric(
            &mut output,
            "arm_hypervisor_system_load_5min",
            "System load average (5 minutes)",
            "gauge",
            load_avg.five.to_string(),
        );
        add_metric(
            &mut output,
            "arm_hypervisor_system_load_15min",
            "System load average (15 minutes)",
            "gauge",
            load_avg.fifteen.to_string(),
        );
    }

    if let Ok(mem_info) = sys_info::mem_info() {
        add_metric(
            &mut output,
            "arm_hypervisor_memory_total_kb",
            "Total system memory in KB",
            "gauge",
            mem_info.total.to_string(),
        );
        add_metric(
            &mut output,
            "arm_hypervisor_memory_free_kb",
            "Free system memory in KB",
            "gauge",
            mem_info.free.to_string(),
        );
        add_metric(
            &mut output,
            "arm_hypervisor_memory_available_kb",
            "Available system memory in KB",
            "gauge",
            mem_info.avail.to_string(),
        );
    }

    if let Ok(disk_info) = sys_info::disk_info() {
        add_metric(
            &mut output,
            "arm_hypervisor_disk_total_kb",
            "Total disk space in KB",
            "gauge",
            disk_info.total.to_string(),
        );
        add_metric(
            &mut output,
            "arm_hypervisor_disk_free_kb",
            "Free disk space in KB",
            "gauge",
            disk_info.free.to_string(),
        );
    }

    add_metric(
        &mut output,
        "arm_hypervisor_cpu_count",
        "Number of CPU cores",
        "gauge",
        num_cpus::get().to_string(),
    );

    // Container metrics
    if let Ok(containers) = ContainerManager::list().await {
        add_metric(
            &mut output,
            "arm_hypervisor_containers_total",
            "Total number of containers",
            "gauge",
            containers.len().to_string(),
        );

        let mut running_count = 0;
        let mut stopped_count = 0;
        let mut error_count = 0;

        for container_name in containers {
            if let Ok(status) = ContainerManager::status(&container_name).await {
                match status {
                    ContainerStatus::Running => running_count += 1,
                    ContainerStatus::Stopped => stopped_count += 1,
                    ContainerStatus::Error => error_count += 1,
                    _ => {}
                }
            }
        }

        add_metric(
            &mut output,
            "arm_hypervisor_containers_running",
            "Number of running containers",
            "gauge",
            running_count.to_string(),
        );
        add_metric(
            &mut output,
            "arm_hypervisor_containers_stopped",
            "Number of stopped containers",
            "gauge",
            stopped_count.to_string(),
        );
        add_metric(
            &mut output,
            "arm_hypervisor_containers_error",
            "Number of containers in error state",
            "gauge",
            error_count.to_string(),
        );
    }

    // Network metrics
    if let Ok(bridges) = BridgeManager::list().await {
        add_metric(
            &mut output,
            "arm_hypervisor_bridges_total",
            "Total number of network bridges",
            "gauge",
            bridges.len().to_string(),
        );
    }

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(output)
}
