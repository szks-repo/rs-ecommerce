use axum::{Json, http::StatusCode};

use crate::rpc::json::{ConnectError, ErrorCode};

#[derive(Debug)]
pub enum IdentityError {
    InvalidArgument(String),
    NotFound(String),
    AlreadyExists(String),
    FailedPrecondition(String),
    PermissionDenied(String),
    Unauthenticated(String),
    Internal(String),
}

pub type IdentityResult<T> = Result<T, IdentityError>;

impl IdentityError {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        IdentityError::InvalidArgument(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        IdentityError::NotFound(message.into())
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        IdentityError::AlreadyExists(message.into())
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        IdentityError::PermissionDenied(message.into())
    }

    pub fn failed_precondition(message: impl Into<String>) -> Self {
        IdentityError::FailedPrecondition(message.into())
    }

    pub fn unauthenticated(message: impl Into<String>) -> Self {
        IdentityError::Unauthenticated(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        IdentityError::Internal(message.into())
    }

    pub fn into_connect(self) -> (StatusCode, Json<ConnectError>) {
        match self {
            IdentityError::InvalidArgument(message) => (
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: ErrorCode::InvalidArgument,
                    message,
                }),
            ),
            IdentityError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ConnectError {
                    code: ErrorCode::NotFound,
                    message,
                }),
            ),
            IdentityError::AlreadyExists(message) => (
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: ErrorCode::AlreadyExists,
                    message,
                }),
            ),
            IdentityError::FailedPrecondition(message) => (
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: ErrorCode::FailedPrecondition,
                    message,
                }),
            ),
            IdentityError::PermissionDenied(message) => (
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: ErrorCode::PermissionDenied,
                    message,
                }),
            ),
            IdentityError::Unauthenticated(message) => (
                StatusCode::UNAUTHORIZED,
                Json(ConnectError {
                    code: ErrorCode::Unauthenticated,
                    message,
                }),
            ),
            IdentityError::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: ErrorCode::Internal,
                    message,
                }),
            ),
        }
    }
}

impl From<sqlx::Error> for IdentityError {
    fn from(err: sqlx::Error) -> Self {
        IdentityError::Internal(format!("db error: {}", err))
    }
}

impl From<(StatusCode, Json<ConnectError>)> for IdentityError {
    fn from(value: (StatusCode, Json<ConnectError>)) -> Self {
        let (status, Json(err)) = value;
        match err.code {
            ErrorCode::InvalidArgument => IdentityError::InvalidArgument(err.message),
            ErrorCode::NotFound => IdentityError::NotFound(err.message),
            ErrorCode::AlreadyExists => IdentityError::AlreadyExists(err.message),
            ErrorCode::FailedPrecondition => IdentityError::FailedPrecondition(err.message),
            ErrorCode::PermissionDenied => IdentityError::PermissionDenied(err.message),
            ErrorCode::Unauthenticated => IdentityError::Unauthenticated(err.message),
            ErrorCode::Internal => IdentityError::Internal(err.message),
            _ => match status {
                StatusCode::BAD_REQUEST => IdentityError::InvalidArgument(err.message),
                StatusCode::NOT_FOUND => IdentityError::NotFound(err.message),
                StatusCode::CONFLICT => IdentityError::AlreadyExists(err.message),
                StatusCode::FORBIDDEN => IdentityError::PermissionDenied(err.message),
                StatusCode::UNAUTHORIZED => IdentityError::Unauthenticated(err.message),
                _ => IdentityError::Internal(err.message),
            },
        }
    }
}
