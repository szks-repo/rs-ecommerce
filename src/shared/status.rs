use crate::pb::pb;
use crate::rpc::json::ConnectError;
use axum::{Json, http::StatusCode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductStatus {
    Active,
    Inactive,
    Draft,
}

impl ProductStatus {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        match value {
            "active" => Ok(ProductStatus::Active),
            "inactive" => Ok(ProductStatus::Inactive),
            "draft" => Ok(ProductStatus::Draft),
            _ => Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "product.status must be active|inactive|draft".to_string(),
                }),
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ProductStatus::Active => "active",
            ProductStatus::Inactive => "inactive",
            ProductStatus::Draft => "draft",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariantStatus {
    Active,
    Inactive,
}

impl VariantStatus {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        match value {
            "active" => Ok(VariantStatus::Active),
            "inactive" => Ok(VariantStatus::Inactive),
            _ => Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "variant.status must be active|inactive".to_string(),
                }),
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            VariantStatus::Active => "active",
            VariantStatus::Inactive => "inactive",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FulfillmentType {
    Physical,
    Digital,
}

impl FulfillmentType {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        match value {
            "" | "physical" => Ok(FulfillmentType::Physical),
            "digital" => Ok(FulfillmentType::Digital),
            _ => Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "fulfillment_type must be physical|digital".to_string(),
                }),
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FulfillmentType::Physical => "physical",
            FulfillmentType::Digital => "digital",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CartStatus {
    Active,
    Ordered,
}

impl CartStatus {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        match value {
            "active" => Ok(CartStatus::Active),
            "ordered" => Ok(CartStatus::Ordered),
            _ => Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "cart.status must be active|ordered".to_string(),
                }),
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CartStatus::Active => "active",
            CartStatus::Ordered => "ordered",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CartItemStatus {
    Active,
    Removed,
}

impl CartItemStatus {
    pub fn parse(value: &str) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        match value {
            "active" => Ok(CartItemStatus::Active),
            "removed" => Ok(CartItemStatus::Removed),
            _ => Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "cart_item.status must be active|removed".to_string(),
                }),
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CartItemStatus::Active => "active",
            CartItemStatus::Removed => "removed",
        }
    }
}

pub fn order_status_to_string(status: i32) -> Option<&'static str> {
    match pb::OrderStatus::try_from(status).ok()? {
        pb::OrderStatus::PendingPayment => Some("pending_payment"),
        pb::OrderStatus::PendingShipment => Some("pending_shipment"),
        pb::OrderStatus::Shipped => Some("shipped"),
        pb::OrderStatus::Completed => Some("completed"),
        pb::OrderStatus::Canceled => Some("canceled"),
        pb::OrderStatus::Unspecified => None,
    }
}

pub fn order_status_from_string(status: String) -> i32 {
    match status.as_str() {
        "pending_payment" => pb::OrderStatus::PendingPayment as i32,
        "pending_shipment" => pb::OrderStatus::PendingShipment as i32,
        "shipped" => pb::OrderStatus::Shipped as i32,
        "completed" => pb::OrderStatus::Completed as i32,
        "canceled" => pb::OrderStatus::Canceled as i32,
        _ => pb::OrderStatus::Unspecified as i32,
    }
}

pub fn payment_method_from_string(method: String) -> i32 {
    PaymentMethod::from_str(method.as_str())
        .map(|value| value.to_pb())
        .unwrap_or(pb::PaymentMethod::Unspecified as i32)
}

pub fn payment_method_to_string(method: i32) -> Option<&'static str> {
    PaymentMethod::from_pb(method)
        .ok()
        .map(|value| value.as_str())
}

pub fn shipment_status_to_string(status: i32) -> &'static str {
    match pb::ShipmentStatus::try_from(status).ok() {
        Some(pb::ShipmentStatus::Pending) => "pending",
        Some(pb::ShipmentStatus::Shipped) => "shipped",
        Some(pb::ShipmentStatus::Delivered) => "delivered",
        Some(pb::ShipmentStatus::Canceled) => "canceled",
        _ => "pending",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentMethod {
    BankTransfer,
    Cod,
}

impl PaymentMethod {
    pub fn from_pb(value: i32) -> Result<Self, (StatusCode, Json<ConnectError>)> {
        match pb::PaymentMethod::try_from(value).ok() {
            Some(pb::PaymentMethod::BankTransfer) => Ok(PaymentMethod::BankTransfer),
            Some(pb::PaymentMethod::Cod) => Ok(PaymentMethod::Cod),
            _ => Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "payment_method is required".to_string(),
                }),
            )),
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "bank_transfer" => Some(PaymentMethod::BankTransfer),
            "cod" => Some(PaymentMethod::Cod),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentMethod::BankTransfer => "bank_transfer",
            PaymentMethod::Cod => "cod",
        }
    }

    pub fn to_pb(&self) -> i32 {
        match self {
            PaymentMethod::BankTransfer => pb::PaymentMethod::BankTransfer as i32,
            PaymentMethod::Cod => pb::PaymentMethod::Cod as i32,
        }
    }
}
