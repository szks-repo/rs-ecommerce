use axum::{
    Json,
    body::Bytes,
    http::{HeaderMap, StatusCode},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::pb::pb;

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidArgument,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    Unauthenticated,
    FailedPrecondition,
    Internal,
    UnsupportedMediaType,
}

#[derive(Debug, Serialize)]
pub struct ConnectError {
    pub code: ErrorCode,
    pub message: String,
}

pub fn parse_json_body(headers: &HeaderMap, body: Bytes) -> Result<Value, (StatusCode, Json<ConnectError>)> {
    if !is_json_content_type(headers) {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::UnsupportedMediaType,
                message: "content-type must be application/json".to_string(),
            }),
        ));
    }

    serde_json::from_slice::<Value>(&body).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: format!("invalid json: {}", err),
            }),
        )
    })
}

pub fn parse_request<T: DeserializeOwned>(
    headers: &HeaderMap,
    body: Bytes,
) -> Result<T, (StatusCode, Json<ConnectError>)> {
    let value = parse_json_body(headers, body)?;
    serde_json::from_value::<T>(value).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: format!("invalid request: {}", err),
            }),
        )
    })
}

pub fn require_tenant_id(tenant: Option<pb::TenantContext>) -> Result<String, (StatusCode, Json<ConnectError>)> {
    match tenant.and_then(|t| {
        if t.tenant_id.is_empty() {
            None
        } else {
            Some(t.tenant_id)
        }
    }) {
        Some(id) => Ok(id),
        None => Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "tenant.tenant_id is required".to_string(),
            }),
        )),
    }
}

pub fn require_store_id(store: Option<pb::StoreContext>) -> Result<String, (StatusCode, Json<ConnectError>)> {
    match store.and_then(|s| if s.store_id.is_empty() { None } else { Some(s.store_id) }) {
        Some(id) => Ok(id),
        None => Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "store.store_id is required".to_string(),
            }),
        )),
    }
}

fn is_json_content_type(headers: &HeaderMap) -> bool {
    let Some(content_type) = headers.get(axum::http::header::CONTENT_TYPE) else {
        return false;
    };
    let Ok(content_type) = content_type.to_str() else {
        return false;
    };
    content_type.starts_with("application/json")
}
