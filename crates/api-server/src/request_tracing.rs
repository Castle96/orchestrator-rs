/// Request tracing middleware with correlation ID support
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::observability::MetricsCollector;

/// Middleware for adding correlation IDs and request tracing
pub struct RequestTracing {
    metrics: Arc<MetricsCollector>,
}

impl RequestTracing {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self { metrics }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestTracing
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestTracingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestTracingMiddleware {
            service,
            metrics: self.metrics.clone(),
        }))
    }
}

pub struct RequestTracingMiddleware<S> {
    service: S,
    metrics: Arc<MetricsCollector>,
}

impl<S, B> Service<ServiceRequest> for RequestTracingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Record request
        self.metrics.record_request();

        // Generate or extract correlation ID
        let correlation_id = req
            .headers()
            .get("X-Correlation-ID")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);

        // Store correlation ID in request extensions
        req.extensions_mut().insert(correlation_id);

        let method = req.method().to_string();
        let path = req.path().to_string();
        let start = std::time::Instant::now();

        info!(
            correlation_id = %correlation_id,
            method = %method,
            path = %path,
            "Request started"
        );

        let metrics = self.metrics.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await;
            let duration = start.elapsed();

            match &res {
                Ok(response) => {
                    let status = response.status();

                    if status.is_client_error() || status.is_server_error() {
                        metrics.record_error();
                        warn!(
                            correlation_id = %correlation_id,
                            method = %method,
                            path = %path,
                            status = %status.as_u16(),
                            duration_ms = %duration.as_millis(),
                            "Request failed"
                        );
                    } else {
                        info!(
                            correlation_id = %correlation_id,
                            method = %method,
                            path = %path,
                            status = %status.as_u16(),
                            duration_ms = %duration.as_millis(),
                            "Request completed"
                        );
                    }
                }
                Err(err) => {
                    metrics.record_error();
                    warn!(
                        correlation_id = %correlation_id,
                        method = %method,
                        path = %path,
                        error = %err,
                        duration_ms = %duration.as_millis(),
                        "Request error"
                    );
                }
            }

            res
        })
    }
}
