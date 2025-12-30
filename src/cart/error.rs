use axum::{Json, http::StatusCode};

use crate::rpc::json::{ConnectError, ErrorCode};

#[derive(Debug)]
pub enum CartError {
    InvalidArgument(String),
    NotFound(String),
    AlreadyExists(String),
    FailedPrecondition(String),
    PermissionDenied(String),
    Unauthenticated(String),
    Internal(String),
}

pub type CartResult<T> = Result<T, CartError>;

impl CartError {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        CartError::InvalidArgument(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        CartError::NotFound(message.into())
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        CartError::AlreadyExists(message.into())
    }

    pub fn failed_precondition(message: impl Into<String>) -> Self {
        CartError::FailedPrecondition(message.into())
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        CartError::PermissionDenied(message.into())
    }

    pub fn unauthenticated(message: impl Into<String>) -> Self {
        CartError::Unauthenticated(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        CartError::Internal(message.into())
    }

    pub fn into_connect(self) -> (StatusCode, Json<ConnectError>) {
        match self {
            CartError::InvalidArgument(message) => (
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: ErrorCode::InvalidArgument,
                    message,
                }),
            ),
            CartError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ConnectError {
                    code: ErrorCode::NotFound,
                    message,
                }),
            ),
            CartError::AlreadyExists(message) => (
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: ErrorCode::AlreadyExists,
                    message,
                }),
            ),
            CartError::FailedPrecondition(message) => (
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: ErrorCode::FailedPrecondition,
                    message,
                }),
            ),
            CartError::PermissionDenied(message) => (
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: ErrorCode::PermissionDenied,
                    message,
                }),
            ),
            CartError::Unauthenticated(message) => (
                StatusCode::UNAUTHORIZED,
                Json(ConnectError {
                    code: ErrorCode::Unauthenticated,
                    message,
                }),
            ),
            CartError::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: ErrorCode::Internal,
                    message,
                }),
            ),
        }
    }
}

impl From<sqlx::Error> for CartError {
    fn from(err: sqlx::Error) -> Self {
        CartError::Internal(format!("db error: {}", err))
    }
}

impl From<(StatusCode, Json<ConnectError>)> for CartError {
    fn from(value: (StatusCode, Json<ConnectError>)) -> Self {
        let (status, Json(err)) = value;
        match err.code {
            ErrorCode::InvalidArgument => CartError::InvalidArgument(err.message),
            ErrorCode::NotFound => CartError::NotFound(err.message),
            ErrorCode::AlreadyExists => CartError::AlreadyExists(err.message),
            ErrorCode::FailedPrecondition => CartError::FailedPrecondition(err.message),
            ErrorCode::PermissionDenied => CartError::PermissionDenied(err.message),
            ErrorCode::Unauthenticated => CartError::Unauthenticated(err.message),
            ErrorCode::Internal => CartError::Internal(err.message),
            _ => match status {
                StatusCode::BAD_REQUEST => CartError::InvalidArgument(err.message),
                StatusCode::NOT_FOUND => CartError::NotFound(err.message),
                StatusCode::CONFLICT => CartError::AlreadyExists(err.message),
                StatusCode::FORBIDDEN => CartError::PermissionDenied(err.message),
                StatusCode::UNAUTHORIZED => CartError::Unauthenticated(err.message),
                _ => CartError::Internal(err.message),
            },
        }
    }
}
