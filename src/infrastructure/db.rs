use axum::{Json, http::StatusCode};

use crate::{AppState, rpc::json::ConnectError};

pub async fn ping(state: &AppState) -> Result<(), (StatusCode, Json<ConnectError>)> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::Internal,
                    message: format!("db error: {}", err),
                }),
            )
        })?;
    Ok(())
}

pub fn error(err: sqlx::Error) -> (StatusCode, Json<ConnectError>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ConnectError {
            code: crate::rpc::json::ErrorCode::Internal,
            message: format!("db error: {}", err),
        }),
    )
}
