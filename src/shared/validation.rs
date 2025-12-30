use axum::{Json, http::StatusCode};

use crate::rpc::json::ConnectError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreCode(String);


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Email(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Phone(String);

impl StoreCode {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_code is required".to_string(),
                }),
            ));
        }
        if normalized.len() > 64 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_code must be 64 chars or less".to_string(),
                }),
            ));
        }
        if !normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_code must be alphanumeric or -_. only".to_string(),
                }),
            ));
        }
        Ok(Self(normalized.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}


impl Email {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "email is required".to_string(),
                }),
            ));
        }
        if !normalized.contains('@') {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "email is invalid".to_string(),
                }),
            ));
        }
        Ok(Self(normalized.to_string()))
    }

    pub fn parse_optional(value: &str) -> Result<Option<Self>, (StatusCode, Json<ConnectError>)> {
        if value.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self::parse(value)?))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Phone {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "phone is required".to_string(),
                }),
            ));
        }
        if normalized.len() > 32 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "phone must be 32 chars or less".to_string(),
                }),
            ));
        }
        Ok(Self(normalized.to_string()))
    }

    pub fn parse_optional(value: &str) -> Result<Option<Self>, (StatusCode, Json<ConnectError>)> {
        if value.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self::parse(value)?))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
