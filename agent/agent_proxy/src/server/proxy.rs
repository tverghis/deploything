use std::sync::{Arc, RwLock};

use axum::{
    Router,
    extract::{OriginalUri, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::any,
};
use tokio::net::TcpListener;
use tracing::{info, warn};

use crate::route::RouteTable;

/// `ReverseProxy` is a simple web server that accepts any incoming request,
/// and uses a set of rules to decide where to proxy that request.
#[derive(Debug)]
pub struct ReverseProxy;

#[derive(Debug, Default, Clone)]
pub struct ProxyState {
    route_table: Arc<RwLock<RouteTable>>,
}

impl ReverseProxy {
    pub fn new() -> Self {
        Self
    }

    pub async fn serve(&self, listener: TcpListener) {
        let state = ProxyState::default();
        let app = Router::new()
            .route("/{*rest}", any(proxy_request))
            .with_state(state);

        info!(
            "Reverse-proxy listening on port {}",
            listener.local_addr().unwrap().port()
        );

        axum::serve(listener, app).await.unwrap();
    }
}

async fn proxy_request(
    uri: OriginalUri,
    headers: HeaderMap,
    State(state): State<ProxyState>,
) -> impl IntoResponse {
    let rt = state.route_table.read().unwrap();

    let hostname = headers.get("host").unwrap().to_str().unwrap();
    let path = uri.to_string();

    match rt.route(hostname, &path) {
        Some(svc) => {
            info!("Routing request to {svc:?}");
        }
        None => {
            warn!("No suitable route entry found.")
        }
    };

    (StatusCode::OK, "Hello world!")
}
