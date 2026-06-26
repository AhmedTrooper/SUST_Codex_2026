use std::env;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

pub struct AppConfig {
    pub port: u16,
    pub db_pool: Option<PgPool>,
    pub redis_client: Option<redis::Client>,
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
                    if let Err(e) = sqlx::query(
                        "CREATE TABLE IF NOT EXISTS analyzed_tickets (
                            ticket_id VARCHAR(255) PRIMARY KEY,
                            complaint TEXT NOT NULL,
                            language VARCHAR(50),
                            channel VARCHAR(50),
                            user_type VARCHAR(50),
                            campaign_context VARCHAR(100),
                            relevant_transaction_id VARCHAR(255),
                            evidence_verdict VARCHAR(50) NOT NULL,
                            case_type VARCHAR(100) NOT NULL,
                            severity VARCHAR(50) NOT NULL,
                            department VARCHAR(100) NOT NULL,
                            agent_summary TEXT NOT NULL,
                            recommended_next_action TEXT NOT NULL,
                            customer_reply TEXT NOT NULL,
                            human_review_required BOOLEAN NOT NULL,
                            confidence DOUBLE PRECISION,
                            reason_codes JSONB,
                            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                        );"
                    ).execute(&pool).await {
                        tracing::error!("Failed to initialize database table: {e}");
                    } else {
                        info!("analyzed_tickets table verified/created.");
                    }
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

        let redis_client = if let Ok(redis_url) = env::var("REDIS_URL") {
            info!("Connecting to Redis...");
            match redis::Client::open(redis_url) {
                Ok(client) => {
                    info!("Redis client initialized.");
                    Some(client)
                }
                Err(e) => {
                    tracing::warn!("Failed to open Redis client: {e}");
                    None
                }
            }
        } else {
            info!("No REDIS_URL set. Running without Redis cache.");
            None
        };

        Self { port, db_pool, redis_client }
    }
}
