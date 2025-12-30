use axum::{Json, http::StatusCode};
use serde_json::Value;

use crate::{AppState, rpc::{json::ConnectError, request_context}, shared::audit_action::AuditAction};

pub struct AuditInput {
    pub tenant_id: String,
    pub actor_id: Option<String>,
    pub actor_type: String,
    pub action: AuditAction,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub request_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub before_json: Option<Value>,
    pub after_json: Option<Value>,
    pub metadata_json: Option<Value>,
}

pub async fn record(
    state: &AppState,
    input: AuditInput,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let mut request_id = input.request_id;
    let mut ip_address = input.ip_address;
    let mut user_agent = input.user_agent;
    if let Some(ctx) = request_context::current() {
        if request_id.is_none() {
            request_id = ctx.request_id;
        }
        if ip_address.is_none() {
            ip_address = ctx.ip_address;
        }
        if user_agent.is_none() {
            user_agent = ctx.user_agent;
        }
    }

    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            tenant_id, actor_id, actor_type, action, target_type, target_id,
            request_id, ip_address, user_agent,
            before_json, after_json, metadata_json
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
        "#,
    )
    .bind(uuid::Uuid::parse_str(&input.tenant_id).ok())
    .bind(input.actor_id)
    .bind(input.actor_type)
    .bind(input.action.as_str())
    .bind(input.target_type)
    .bind(input.target_id)
    .bind(request_id)
    .bind(ip_address)
    .bind(user_agent)
    .bind(input.before_json)
    .bind(input.after_json)
    .bind(input.metadata_json)
    .execute(&state.db)
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::Internal,
                message: format!("audit error: {}", err),
            }),
        )
    })?;
    Ok(())
}
