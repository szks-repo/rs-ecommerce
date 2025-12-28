use axum::{Json, http::StatusCode};

use crate::rpc::json::ConnectError;

pub fn nullable_uuid(id: String) -> Option<uuid::Uuid> {
    if id.is_empty() {
        None
    } else {
        uuid::Uuid::parse_str(&id).ok()
    }
}

pub fn parse_uuid(id: &str, field: &str) -> Result<uuid::Uuid, (StatusCode, Json<ConnectError>)> {
    uuid::Uuid::parse_str(id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: format!("{} must be a valid UUID", field),
            }),
        )
    })
}
