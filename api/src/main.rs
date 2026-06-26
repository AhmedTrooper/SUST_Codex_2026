mod config;
mod handlers;
mod models;
mod investigator;

use axum::{routing::{get, post}, Router};
use config::AppConfig;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Option<sqlx::PgPool>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configurations and connect to DB if available
    let config = AppConfig::load().await;

    // Configure CORS allowing any origin
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState {
        db_pool: config.db_pool.clone(),
    };

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/analyze-ticket", post(handlers::analyze_ticket))
        .route("/tickets", get(handlers::list_tickets))
        .with_state(state)
        .layer(cors);

    // Bind to 0.0.0.0 on the configured port
    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
