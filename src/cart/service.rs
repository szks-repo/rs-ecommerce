use axum::{Json, http::StatusCode};

use crate::{AppState, pb::pb, infrastructure::db, rpc::json::ConnectError, shared::ids::parse_uuid};

pub async fn create_cart(
    state: &AppState,
    tenant_id: String,
    req: pb::CreateCartRequest,
) -> Result<pb::Cart, (StatusCode, Json<ConnectError>)> {
    let cart_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO carts (id, tenant_id, customer_id, status)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(cart_id)
    .bind(&tenant_id)
    .bind(parse_uuid(&req.customer_id, "customer_id").ok())
    .bind("active")
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::Cart {
        id: cart_id.to_string(),
        customer_id: String::new(),
        items: Vec::new(),
        total: None,
        status: "active".to_string(),
    })
}
