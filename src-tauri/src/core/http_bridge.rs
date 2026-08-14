use crate::crypto_token;
use axum::{
    extract::Query,
    http::{HeaderName, HeaderValue, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
};

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

pub async fn start_http_server(app_handle: tauri::AppHandle) {
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3004".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:3004".parse::<HeaderValue>().unwrap(),
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),
            "https://modesthumanbrands.com"
                .parse::<HeaderValue>()
                .unwrap(),
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .max_age(Duration::from_secs(86400));

    let pna_layer = SetResponseHeaderLayer::overriding(
        HeaderName::from_static("access-control-allow-private-network"),
        HeaderValue::from_static("true"),
    );

    let hls_dir = std::env::temp_dir().join("msync_hsl_stream");
    let _ = std::fs::create_dir_all(&hls_dir);

    let axum_app = Router::new()
        .route("/api/health", get(health_check))
        .route(
            "/api/certificates",
            get(|| async {
                match crypto_token::list_certificates().await {
                    Ok(certs) => Ok(Json(certs)),
                    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
                }
            }),
        )
        .route(
            "/api/sign-hash",
            post(|Query(q): Query<SignHashQuery>| async move {
                match crypto_token::sign_hash(q.digest_hex, q.cert_index).await {
                    Ok(response) => Ok(Json(response)),
                    Err(e) => Err((StatusCode::BAD_REQUEST, e)),
                }
            }),
        )
        .nest_service("/api/preview", ServeDir::new(hls_dir))
        .with_state(app_handle)
        .layer(pna_layer)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8720));
    if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
        println!("🌐 Hybrid HTTP Bridge listening on http://{}", addr);
        let _ = axum::serve(listener, axum_app).await;
    }
}
