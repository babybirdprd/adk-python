//! Logging middleware

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tower::{Layer, Service};
use tracing::{info, warn};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

/// Logging middleware layer
#[derive(Clone)]
pub struct LoggingLayer;

impl LoggingLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddleware { inner }
    }
}

/// Logging middleware service
#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for LoggingMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let start = Instant::now();
            let method = request.method().clone();
            let uri = request.uri().clone();
            let version = request.version();

            // Extract request ID if present
            let request_id = request
                .headers()
                .get(super::request_id::REQUEST_ID_HEADER)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
                .to_string();

            info!(
                method = %method,
                uri = %uri,
                version = ?version,
                request_id = %request_id,
                "Started processing request"
            );

            // Call the inner service
            let response = inner.call(request).await?;
            
            let duration = start.elapsed();
            let status = response.status();

            // Log the response
            if status.is_success() {
                info!(
                    status = %status,
                    duration_ms = duration.as_millis(),
                    request_id = %request_id,
                    "Request completed successfully"
                );
            } else if status.is_client_error() {
                warn!(
                    status = %status,
                    duration_ms = duration.as_millis(),
                    request_id = %request_id,
                    "Request completed with client error"
                );
            } else {
                warn!(
                    status = %status,
                    duration_ms = duration.as_millis(),
                    request_id = %request_id,
                    "Request completed with server error"
                );
            }

            Ok(response)
        })
    }
}

impl Default for LoggingLayer {
    fn default() -> Self {
        Self::new()
    }
}
