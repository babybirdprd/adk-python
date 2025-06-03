//! Web server and API system

pub mod server;
pub mod handlers;
pub mod websocket;
pub mod middleware;

pub use server::{WebServer, ServerConfig, WebServerBuilder, ServerState};
pub use handlers::*;
pub use websocket::WebSocketHandler;
