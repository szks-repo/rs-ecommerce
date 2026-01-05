use axum::{Json, http::StatusCode};

use crate::{pb::pb, rpc::json::ConnectError};

pub fn money_to_parts(money: Option<pb::Money>) -> Result<(i64, String), (StatusCode, Json<ConnectError>)> {
    let Some(money) = money else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "money is required".to_string(),
            }),
        ));
    };
    if money.currency.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "money.currency is required".to_string(),
            }),
        ));
    }
    Ok((money.amount, money.currency))
}

pub fn money_to_parts_opt(
    money: Option<pb::Money>,
) -> Result<(Option<i64>, Option<String>), (StatusCode, Json<ConnectError>)> {
    let Some(money) = money else {
        return Ok((None, None));
    };
    if money.currency.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "compare_at.currency is required when compare_at is set".to_string(),
            }),
        ));
    }
    Ok((Some(money.amount), Some(money.currency)))
}

pub fn money_from_parts(amount: i64, currency: String) -> pb::Money {
    pb::Money { amount, currency }
}
