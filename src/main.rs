use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use rs_common::{env, telemetry};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

mod auction;
mod audit;
mod cart;
mod customer;
mod identity;
mod infrastructure;
mod order;
mod pb;
mod product;
mod promotion;
mod rpc;
mod setup;
mod shared;
mod store_settings;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    telemetry::init_tracing("rs-ecommerce");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    sqlx::migrate!().run(&db).await?;

    let search_backend =
        std::env::var("SEARCH_BACKEND").unwrap_or_else(|_| "meili".to_string());
    let search = match search_backend.as_str() {
        "none" => infrastructure::search::SearchService::none(),
        "meili" | "meilisearch" => {
            let meili_url = std::env::var("MEILI_URL").expect("MEILI_URL is required");
            let meili_key = std::env::var("MEILI_MASTER_KEY").ok();
            infrastructure::search::SearchService::meilisearch(
                &meili_url,
                meili_key.as_deref(),
                "products",
            )
        }
        "opensearch" => {
            let os_url = std::env::var("OPENSEARCH_URL").expect("OPENSEARCH_URL is required");
            let os_index =
                std::env::var("OPENSEARCH_INDEX").unwrap_or_else(|_| "products".to_string());
            infrastructure::search::SearchService::opensearch(&os_url, &os_index)
        }
        other => {
            panic!("unknown SEARCH_BACKEND: {}", other);
        }
    };
    search.ensure_settings().await.expect("search settings");

    let app_state = AppState { db, search };
    let scheduler_state = app_state.clone();
    tokio::spawn(async move {
        let batch_size = env::env_usize("AUCTION_WORKER_BATCH_SIZE", 50) as i64;
        let sleep_ms = env::env_u64("AUCTION_WORKER_SLEEP_MS", 1000);
        let oneshot = env::env_bool("AUCTION_WORKER_ONESHOT", false);
        loop {
            match auction::service::run_scheduled_auctions(&scheduler_state, batch_size).await {
                Ok(done) => {
                    if done > 0 {
                        tracing::info!(done, "auction auto-bid scheduler executed");
                    }
                }
                Err(err) => {
                    let message = err.1.0.message.clone();
                    tracing::warn!(error = %message, "auction auto-bid scheduler failed");
                }
            }
            if oneshot {
                break;
            }
            tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        }
    });
    let app = Router::new()
        .route("/health", get(health))
        .with_state(app_state.clone())
        .merge(rpc::router(app_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("bind 0.0.0.0:8080");
    tracing::info!(
        "listening on {}",
        listener.local_addr().expect("local addr")
    );
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub search: infrastructure::search::SearchService,
}

async fn health(
    State(state): State<AppState>,
) -> Result<&'static str, (StatusCode, Json<rpc::json::ConnectError>)> {
    infrastructure::db::ping(&state).await?;
    Ok("ok")
}
