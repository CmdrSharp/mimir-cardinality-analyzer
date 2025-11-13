use crate::{config::Config, metrics::METRICS_HANDLE};
use axum::{Router, response::IntoResponse, routing::get};
use hyper::StatusCode;
use std::net::SocketAddr;

/// Creates an Axum Web Server
pub async fn create_server(config: Config) {
    tracing::info!("Starting the web server");

    let app = create_router();

    let addr: SocketAddr = format!("{}:{}", config.http.host, config.http.port)
        .parse()
        .expect("Unable to parse address");

    tracing::info!("Listening on {}", addr);

    axum_server::bind(addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

/// Create the router for the application
fn create_router() -> Router {
    Router::new()
        .route("/alive", get(alive))
        .route("/metrics", get(metrics))
}

/// This is the handler for the /alive path
async fn alive() -> StatusCode {
    StatusCode::OK
}

/// This is the handler for the /metrics path
#[tracing::instrument]
async fn metrics() -> impl IntoResponse {
    match METRICS_HANDLE.get().unwrap() {
        Some(handle) => (StatusCode::OK, handle.render()),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get the metrics handle".to_string(),
        ),
    }
}
