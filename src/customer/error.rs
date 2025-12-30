use axum::{Json, http::StatusCode};

use crate::rpc::json::{ConnectError, ErrorCode};

#[derive(Debug)]
pub enum CustomerError {
    InvalidArgument(String),
    NotFound(String),
    AlreadyExists(String),
    PermissionDenied(String),
    Unauthenticated(String),
    Internal(String),
}

pub type CustomerResult<T> = Result<T, CustomerError>;

impl CustomerError {
    pub fn into_connect(self) -> (StatusCode, Json<ConnectError>) {
        match self {
            CustomerError::InvalidArgument(message) => (
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: ErrorCode::InvalidArgument,
                    message,
                }),
            ),
            CustomerError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ConnectError {
                    code: ErrorCode::NotFound,
                    message,
                }),
            ),
            CustomerError::AlreadyExists(message) => (
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: ErrorCode::AlreadyExists,
                    message,
                }),
            ),
            CustomerError::PermissionDenied(message) => (
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: ErrorCode::PermissionDenied,
                    message,
                }),
            ),
            CustomerError::Unauthenticated(message) => (
                StatusCode::UNAUTHORIZED,
                Json(ConnectError {
                    code: ErrorCode::Unauthenticated,
                    message,
                }),
            ),
            CustomerError::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: ErrorCode::Internal,
                    message,
                }),
            ),
        }
    }
}

impl From<sqlx::Error> for CustomerError {
    fn from(err: sqlx::Error) -> Self {
        CustomerError::Internal(format!("db error: {}", err))
    }
}

impl From<(StatusCode, Json<ConnectError>)> for CustomerError {
    fn from(value: (StatusCode, Json<ConnectError>)) -> Self {
        let (status, Json(err)) = value;
        match status {
            StatusCode::BAD_REQUEST => CustomerError::InvalidArgument(err.message),
            StatusCode::NOT_FOUND => CustomerError::NotFound(err.message),
            StatusCode::CONFLICT => CustomerError::AlreadyExists(err.message),
            StatusCode::FORBIDDEN => CustomerError::PermissionDenied(err.message),
            StatusCode::UNAUTHORIZED => CustomerError::Unauthenticated(err.message),
            _ => CustomerError::Internal(err.message),
        }
    }
}
