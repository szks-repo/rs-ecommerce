use axum::{Json, http::StatusCode};
use serde_json::Value;
use sqlx::{Postgres, Transaction};

use crate::{AppState, rpc::json::ConnectError, rpc::request_context};

pub struct OutboxEventInput {
    pub tenant_id: String,
    pub store_id: Option<String>,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload_json: Value,
}

pub async fn enqueue(
    state: &AppState,
    input: OutboxEventInput,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let event_id = uuid::Uuid::new_v4();
    let idempotency_key = build_idempotency_key(&input);
    let store_uuid = input
        .store_id
        .and_then(|id| uuid::Uuid::parse_str(&id).ok());
    let tenant_uuid = uuid::Uuid::parse_str(&input.tenant_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "tenant_id is invalid".to_string(),
            }),
        )
    })?;

    sqlx::query(
        r#"
        INSERT INTO outbox_events
            (id, tenant_id, store_id, aggregate_type, aggregate_id, event_type, payload_json, status, idempotency_key)
        VALUES ($1,$2,$3,$4,$5,$6,$7,'pending',$8)
        ON CONFLICT (tenant_id, idempotency_key) DO NOTHING
        "#,
    )
    .bind(event_id)
    .bind(tenant_uuid)
    .bind(store_uuid)
    .bind(&input.aggregate_type)
    .bind(&input.aggregate_id)
    .bind(&input.event_type)
    .bind(&input.payload_json)
    .bind(idempotency_key)
    .execute(&state.db)
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::Internal,
                message: format!("outbox enqueue failed: {}", err),
            }),
        )
    })?;

    Ok(())
}

pub async fn enqueue_tx(
    tx: &mut Transaction<'_, Postgres>,
    input: OutboxEventInput,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let event_id = uuid::Uuid::new_v4();
    let idempotency_key = build_idempotency_key(&input);
    let store_uuid = input
        .store_id
        .and_then(|id| uuid::Uuid::parse_str(&id).ok());
    let tenant_uuid = uuid::Uuid::parse_str(&input.tenant_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "tenant_id is invalid".to_string(),
            }),
        )
    })?;

    sqlx::query(
        r#"
        INSERT INTO outbox_events
            (id, tenant_id, store_id, aggregate_type, aggregate_id, event_type, payload_json, status, idempotency_key)
        VALUES ($1,$2,$3,$4,$5,$6,$7,'pending',$8)
        ON CONFLICT (tenant_id, idempotency_key) DO NOTHING
        "#,
    )
    .bind(event_id)
    .bind(tenant_uuid)
    .bind(store_uuid)
    .bind(&input.aggregate_type)
    .bind(&input.aggregate_id)
    .bind(&input.event_type)
    .bind(&input.payload_json)
    .bind(idempotency_key)
    .execute(tx.as_mut())
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::Internal,
                message: format!("outbox enqueue failed: {}", err),
            }),
        )
    })?;

    Ok(())
}

fn build_idempotency_key(input: &OutboxEventInput) -> String {
    if let Some(ctx) = request_context::current()
        && let Some(request_id) = ctx.request_id {
            return format!(
                "{}:{}:{}",
                input.event_type, input.aggregate_type, request_id
            );
        }
    uuid::Uuid::new_v4().to_string()
}
