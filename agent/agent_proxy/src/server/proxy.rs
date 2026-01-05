use axum::{Router, routing::any};
use tokio::net::TcpListener;
use tracing::info;

use crate::route::RouteTable;

/// `ReverseProxy` is a simple web server that accepts any incoming request,
/// and uses a set of rules to decide where to proxy that request.
#[derive(Debug)]
pub struct ReverseProxy {
    route_table: RouteTable,
}

impl ReverseProxy {
    pub fn new() -> Self {
        Self {
            route_table: RouteTable::default(),
        }
    }

    pub async fn serve(&self, listener: TcpListener) {
        let app = Router::new().route("/{*rest}", any(|| async { "hello world!" }));

        info!(
            "Reverse-proxy listening on port {}",
            listener.local_addr().unwrap().port()
        );

        axum::serve(listener, app).await.unwrap();
    }
}
