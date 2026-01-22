use actix_web::{middleware::Logger, web, App, HttpServer};
use std::path::Path;

mod config;
mod handlers;
mod middleware;
mod routes;

use config::AppConfig;
use middleware::{RequestLogging, SecurityHeaders, SimpleCors};
use routes::configure_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    let mut app_config = AppConfig::from_env();

    // Try to load from config file if it exists
    let config_paths = [
        "/etc/arm-hypervisor/config.toml",
        "/etc/arm-hypervisor/config.yaml",
        "./config.toml",
        "./config.yaml",
    ];

    for config_path in &config_paths {
        if Path::new(config_path).exists() {
            if let Err(e) = app_config.merge_with_file(config_path) {
                eprintln!("Warning: Failed to load config from {}: {}", config_path, e);
            } else {
                println!("Loaded configuration from: {}", config_path);
                break;
            }
        }
    }

    // Validate configuration
    if let Err(errors) = app_config.validate() {
        eprintln!("Configuration validation failed:");
        for error in errors {
            eprintln!("  - {}", error);
        }
        std::process::exit(1);
    }

    // Initialize logging based on config
    let log_level = &app_config.logging.level;
    let env_filter = format!("api_server={},actix_web=info,tower_http=info", log_level);

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    tracing::info!("Starting ARM Hypervisor API server");
    tracing::info!(
        "Server config: {}:{}",
        app_config.server.host,
        app_config.server.port
    );
    tracing::info!(
        "Authentication enabled: {}",
        app_config.security.auth_enabled
    );
    
    // Override JWT secret from environment if provided
    if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
        tracing::info!("JWT secret loaded from environment variable");
        app_config.security.jwt_secret = Some(jwt_secret);
    }
    
    if app_config.security.auth_enabled && app_config.security.jwt_secret.is_none() {
        eprintln!("CRITICAL ERROR: Authentication is enabled but no JWT secret is configured!");
        eprintln!("Set JWT_SECRET environment variable or configure it in config file.");
        std::process::exit(1);
    }
    
    if app_config.server.tls.is_some() {
        tracing::info!("TLS is enabled");
    } else {
        tracing::warn!("TLS is NOT enabled - this should only be used in development!");
    }

    // Create shared app data
    let server_config = app_config.server.clone();
    let _security_config = app_config.security.clone();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_config.clone()))
            .wrap(Logger::default())
            .wrap(SecurityHeaders)
            .wrap(RequestLogging)
            .wrap(SimpleCors)
            .configure(configure_routes)
    });

    // Configure server based on config
    let bind_address = (server_config.host.as_str(), server_config.port);
    
    let mut server = if let Some(ref tls_config) = server_config.tls {
        // TLS is configured - bind with TLS
        tracing::info!("Binding with TLS using cert: {:?}", tls_config.cert_file);
        
        use rustls::ServerConfig;
        use rustls_pemfile::{certs, pkcs8_private_keys};
        use std::fs::File;
        use std::io::BufReader;
        
        // Load TLS certificate and key
        let cert_file = &mut BufReader::new(File::open(&tls_config.cert_file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to open cert file: {}", e)))?);
        let key_file = &mut BufReader::new(File::open(&tls_config.key_file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to open key file: {}", e)))?);
        
        let cert_chain = certs(cert_file)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to parse cert: {}", e)))?;
        let mut keys = pkcs8_private_keys(key_file)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to parse key: {}", e)))?;
        
        if keys.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, 
                "No private key found in key file"));
        }
        
        let tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, keys.remove(0).into())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to build TLS config: {}", e)))?;
        
        server.bind_rustls_0_23(bind_address, tls_config)?
    } else {
        // No TLS - plain HTTP
        server.bind(bind_address)?
    };

    if let Some(workers) = server_config.workers {
        server = server.workers(workers);
    }

    if let Some(max_conn) = server_config.max_connections {
        server = server.max_connections(max_conn);
    }

    if let Some(keepalive) = server_config.keepalive {
        server = server.keep_alive(std::time::Duration::from_secs(keepalive));
    }

    if let Some(timeout) = server_config.client_timeout {
        server = server.client_request_timeout(std::time::Duration::from_secs(timeout));
    }

    tracing::info!("ARM Hypervisor API server started successfully");
    server.run().await
}
