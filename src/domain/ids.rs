use axum::{Json, http::StatusCode};
use std::fmt;

use crate::rpc::json::ConnectError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TenantId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct StoreId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ProductId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct VariantId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LocationId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SkuId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CartId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CartItemId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CustomerId(uuid::Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct StaffId(uuid::Uuid);

impl TenantId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "tenant_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl StoreId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "store_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl ProductId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "product_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl VariantId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "variant_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl LocationId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "location_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl SkuId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "sku_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl CartId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "cart_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl CartItemId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "cart_item_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl CustomerId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "customer_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl StaffId {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        parse_uuid(value, "staff_id").map(Self)
    }
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for StoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ProductId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for VariantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for LocationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for SkuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for CartId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for CartItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for CustomerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for StaffId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: format!("{} must be a valid UUID", field),
            }),
        )
    })
}
