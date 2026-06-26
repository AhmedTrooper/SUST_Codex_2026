use std::env;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

pub struct AppConfig {
    pub port: u16,
    pub db_pool: Option<PgPool>,
}

impl AppConfig {
    pub async fn load() -> Self {
        // Load .env if present
        let _ = dotenvy::dotenv();

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);

        let db_pool = if let Ok(db_url) = env::var("DATABASE_URL") {
            info!("Connecting to PostgreSQL database...");
            match PgPoolOptions::new()
                .max_connections(5)
                .connect(&db_url)
                .await
            {
                Ok(pool) => {
                    info!("PostgreSQL connection pool initialized.");
                    Some(pool)
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to database: {e}. Falling back to memory.");
                    None
                }
            }
        } else {
            info!("No DATABASE_URL set. Running in-memory mode.");
            None
        };

        Self { port, db_pool }
    }
}
