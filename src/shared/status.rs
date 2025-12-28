use crate::pb::pb;

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
    match method.as_str() {
        "bank_transfer" => pb::PaymentMethod::BankTransfer as i32,
        "cod" => pb::PaymentMethod::Cod as i32,
        _ => pb::PaymentMethod::Unspecified as i32,
    }
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
