// Domain models for Product context (placeholder for richer invariants).
#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub vendor_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub id: String,
    pub product_id: String,
    pub sku: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkuCode(String);

impl SkuCode {
    pub fn parse(
        value: &str,
    ) -> Result<
        Self,
        (
            axum::http::StatusCode,
            axum::Json<crate::rpc::json::ConnectError>,
        ),
    > {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(crate::rpc::json::ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "sku is required".to_string(),
                }),
            ));
        }
        if normalized.len() > 64 {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(crate::rpc::json::ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "sku must be 64 chars or less".to_string(),
                }),
            ));
        }
        if normalized.contains(char::is_whitespace) {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(crate::rpc::json::ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "sku must not contain whitespace".to_string(),
                }),
            ));
        }
        Ok(Self(normalized.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
