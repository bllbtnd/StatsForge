mod config;
mod error;
mod github;
mod params;
mod svg;

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use config::Config;
use error::AppError;
use params::{CardParams, RawParams};

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    http: reqwest::Client,
}

#[tokio::main]
async fn main() {
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let _ = dotenvy::dotenv();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    });

    let state = AppState {
        config: Arc::new(config.clone()),
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client"),
    };

    let app = Router::new()
        .route("/", get(health))
        .route("/card", get(card_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        });

    info!("StatsForge listening on http://{}", addr);
    axum::serve(listener, app).await.expect("server error");
}

async fn health() -> &'static str {
    "ok"
}

async fn card_handler(
    State(state): State<AppState>,
    Query(raw): Query<RawParams>,
) -> Result<impl IntoResponse, AppError> {
    let params = CardParams::from_raw(raw)?;

    let stats = github::fetch_language_stats(
        &state.http,
        &state.config.github_token,
        &params.username,
        params.number_of_languages,
        params.sort_by,
    )
    .await?;

    let svg_body = svg::render(&params, &stats);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/svg+xml".parse().unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=3600, s-maxage=3600".parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, svg_body))
}
