use axum::{Json, http::StatusCode};

use crate::rpc::json::{ConnectError, ErrorCode};

#[derive(Debug)]
pub enum OrderError {
    InvalidArgument(String),
    NotFound(String),
    AlreadyExists(String),
    FailedPrecondition(String),
    PermissionDenied(String),
    Unauthenticated(String),
    Internal(String),
}

pub type OrderResult<T> = Result<T, OrderError>;

impl OrderError {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        OrderError::InvalidArgument(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        OrderError::NotFound(message.into())
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        OrderError::AlreadyExists(message.into())
    }

    pub fn failed_precondition(message: impl Into<String>) -> Self {
        OrderError::FailedPrecondition(message.into())
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        OrderError::PermissionDenied(message.into())
    }

    pub fn unauthenticated(message: impl Into<String>) -> Self {
        OrderError::Unauthenticated(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        OrderError::Internal(message.into())
    }

    pub fn into_connect(self) -> (StatusCode, Json<ConnectError>) {
        match self {
            OrderError::InvalidArgument(message) => (
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: ErrorCode::InvalidArgument,
                    message,
                }),
            ),
            OrderError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ConnectError {
                    code: ErrorCode::NotFound,
                    message,
                }),
            ),
            OrderError::AlreadyExists(message) => (
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: ErrorCode::AlreadyExists,
                    message,
                }),
            ),
            OrderError::FailedPrecondition(message) => (
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: ErrorCode::FailedPrecondition,
                    message,
                }),
            ),
            OrderError::PermissionDenied(message) => (
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: ErrorCode::PermissionDenied,
                    message,
                }),
            ),
            OrderError::Unauthenticated(message) => (
                StatusCode::UNAUTHORIZED,
                Json(ConnectError {
                    code: ErrorCode::Unauthenticated,
                    message,
                }),
            ),
            OrderError::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: ErrorCode::Internal,
                    message,
                }),
            ),
        }
    }
}

impl From<sqlx::Error> for OrderError {
    fn from(err: sqlx::Error) -> Self {
        OrderError::Internal(format!("db error: {}", err))
    }
}

impl From<(StatusCode, Json<ConnectError>)> for OrderError {
    fn from(value: (StatusCode, Json<ConnectError>)) -> Self {
        let (status, Json(err)) = value;
        match err.code {
            ErrorCode::InvalidArgument => OrderError::InvalidArgument(err.message),
            ErrorCode::NotFound => OrderError::NotFound(err.message),
            ErrorCode::AlreadyExists => OrderError::AlreadyExists(err.message),
            ErrorCode::FailedPrecondition => OrderError::FailedPrecondition(err.message),
            ErrorCode::PermissionDenied => OrderError::PermissionDenied(err.message),
            ErrorCode::Unauthenticated => OrderError::Unauthenticated(err.message),
            ErrorCode::Internal => OrderError::Internal(err.message),
            _ => match status {
                StatusCode::BAD_REQUEST => OrderError::InvalidArgument(err.message),
                StatusCode::NOT_FOUND => OrderError::NotFound(err.message),
                StatusCode::CONFLICT => OrderError::AlreadyExists(err.message),
                StatusCode::FORBIDDEN => OrderError::PermissionDenied(err.message),
                StatusCode::UNAUTHORIZED => OrderError::Unauthenticated(err.message),
                _ => OrderError::Internal(err.message),
            },
        }
    }
}
