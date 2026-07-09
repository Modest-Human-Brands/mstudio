use crate::security;
use axum::{
    extract::Query,
    http::{HeaderValue, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    node: String,
}

#[derive(Deserialize)]
struct SignHashQuery {
    digest_hex: String,
    cert_index: usize,
}

async fn health_check() -> Json<HealthResponse> {
    let version = env!("CARGO_PKG_VERSION");

    let node = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "unknown-node".to_string());

    Json(HealthResponse {
        status: "OK",
        version,
        node,
    })
}

pub async fn start_http_server() {
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "https://modesthumanbrands.com"
                .parse::<HeaderValue>()
                .unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/health", get(health_check)) // <-- Register Health Route Here
        .route(
            "/api/certificates",
            get(|| async {
                match security::list_certificates().await {
                    Ok(certs) => Ok(Json(certs)),
                    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
                }
            }),
        )
        .route(
            "/api/sign-hash",
            post(|Query(q): Query<SignHashQuery>| async move {
                match security::sign_hash(q.digest_hex, q.cert_index).await {
                    Ok(sig) => Ok(Json(serde_json::json!({ "signatureHex": sig }))),
                    Err(e) => Err((StatusCode::BAD_REQUEST, e)),
                }
            }),
        )
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8720));
    if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
        println!("🌐 Hybrid HTTP Bridge listening on http://{}", addr);
        let _ = axum::serve(listener, app).await;
    }
}
