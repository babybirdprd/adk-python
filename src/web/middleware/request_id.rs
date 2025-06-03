//! Request ID middleware

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use tower::{Layer, Service};
use uuid::Uuid;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Request ID header name
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Request ID middleware layer
#[derive(Clone)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdMiddleware { inner }
    }
}

/// Request ID middleware service
#[derive(Clone)]
pub struct RequestIdMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for RequestIdMiddleware<S>
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

    fn call(&mut self, mut request: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Generate or extract request ID
            let request_id = request
                .headers()
                .get(REQUEST_ID_HEADER)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            // Add request ID to request headers
            request.headers_mut().insert(
                REQUEST_ID_HEADER,
                HeaderValue::from_str(&request_id).unwrap(),
            );

            // Call the inner service
            let mut response = inner.call(request).await?;

            // Add request ID to response headers
            response.headers_mut().insert(
                REQUEST_ID_HEADER,
                HeaderValue::from_str(&request_id).unwrap(),
            );

            Ok(response)
        })
    }
}

impl Default for RequestIdLayer {
    fn default() -> Self {
        Self::new()
    }
}
