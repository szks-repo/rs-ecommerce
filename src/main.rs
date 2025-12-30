use axum::{routing::get, extract::State, Json, http::StatusCode};
use rs_common::telemetry;
use sqlx::{PgPool, postgres::PgPoolOptions};

mod product;
mod cart;
mod audit;
mod identity;
mod order;
mod promotion;
mod customer;
mod infrastructure;
mod pb;
mod rpc;
mod domain;
mod shared;
mod store_settings;
mod setup;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    telemetry::init_tracing("rs-ecommerce");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    sqlx::migrate!().run(&db).await?;

    let meili_url = std::env::var("MEILI_URL").expect("MEILI_URL is required");
    let meili_key = std::env::var("MEILI_MASTER_KEY").ok();
    let search = infrastructure::search::SearchClient::new(&meili_url, meili_key.as_deref(), "products");
    search.ensure_settings().await.expect("meilisearch settings");

    let app = rpc::router()
        .route("/health", get(health))
        .with_state(AppState { db, search });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("bind 0.0.0.0:8080");
    tracing::info!("listening on {}", listener.local_addr().expect("local addr"));
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub search: infrastructure::search::SearchClient,
}

async fn health(
    State(state): State<AppState>,
) -> Result<&'static str, (StatusCode, Json<rpc::json::ConnectError>)> {
    infrastructure::db::ping(&state).await?;
    Ok("ok")
}
