use axum::{
    Json,
    body::Body,
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{AppState, rpc::actor::AuthContext, rpc::json::ConnectError};
use sqlx::Row;

pub async fn require_active_staff_session(
    State(state): State<AppState>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Response {
    let auth_ctx = req
        .extensions()
        .get::<Option<AuthContext>>()
        .and_then(|v| v.clone());

    let Some(auth) = auth_ctx else {
        return next.run(req).await;
    };

    let Some(session_id) = auth.session_id.clone() else {
        return next.run(req).await;
    };

    let Some(store_id) = auth.store_id.clone() else {
        return error_response(StatusCode::UNAUTHORIZED, "unauthenticated");
    };

    let session_uuid = match uuid::Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::UNAUTHORIZED, "invalid session"),
    };
    let staff_uuid = match uuid::Uuid::parse_str(&auth.actor_id) {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::UNAUTHORIZED, "invalid session"),
    };
    let store_uuid = match uuid::Uuid::parse_str(&store_id) {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::UNAUTHORIZED, "invalid session"),
    };

    let row = sqlx::query(
        r#"
        SELECT revoked_at, expires_at
        FROM store_staff_sessions
        WHERE id = $1 AND store_id = $2 AND staff_id = $3
        "#,
    )
    .bind(session_uuid)
    .bind(store_uuid)
    .bind(staff_uuid)
    .fetch_optional(&state.db)
    .await;

    let Some(row) = row.ok().flatten() else {
        return error_response(StatusCode::UNAUTHORIZED, "unauthenticated");
    };

    let revoked_at: Option<chrono::DateTime<chrono::Utc>> = row.get("revoked_at");
    let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");

    if revoked_at.is_some() {
        return error_response(StatusCode::UNAUTHORIZED, "unauthenticated");
    }
    if let Some(expires_at) = expires_at {
        if expires_at <= chrono::Utc::now() {
            return error_response(StatusCode::UNAUTHORIZED, "unauthenticated");
        }
    }

    let _ = sqlx::query(
        r#"
        UPDATE store_staff_sessions
        SET last_seen_at = now()
        WHERE id = $1
        "#,
    )
    .bind(session_uuid)
    .execute(&state.db)
    .await;

    next.run(req).await
}

fn error_response(status: StatusCode, message: &str) -> Response {
    let code = match status {
        StatusCode::UNAUTHORIZED => crate::rpc::json::ErrorCode::Unauthenticated,
        StatusCode::INTERNAL_SERVER_ERROR => crate::rpc::json::ErrorCode::Internal,
        _ => crate::rpc::json::ErrorCode::PermissionDenied,
    };
    let body = Json(ConnectError {
        code,
        message: message.to_string(),
    });
    (status, body).into_response()
}
