use axum::Json;
use axum::{
    body::Body,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{AppState, rpc::actor::AuthContext, rpc::json::ConnectError};

pub async fn require_permission(
    req: axum::http::Request<Body>,
    next: Next,
    permission_key: &'static str,
) -> Response {
    let auth_ctx = req
        .extensions()
        .get::<Option<AuthContext>>()
        .and_then(|v| v.clone());
    let Some(auth) = auth_ctx else {
        return error_response(StatusCode::UNAUTHORIZED, "unauthenticated");
    };

    if auth.actor_type == "owner" {
        return next.run(req).await;
    }

    let Some(store_id) = auth.store_id.clone() else {
        return error_response(StatusCode::FORBIDDEN, "store_id is required");
    };

    let state = match req.extensions().get::<AppState>().cloned() {
        Some(state) => state,
        None => {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "state missing");
        }
    };

    let staff_uuid = match uuid::Uuid::parse_str(&auth.actor_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(StatusCode::FORBIDDEN, "actor_id is invalid");
        }
    };
    let store_uuid = match uuid::Uuid::parse_str(&store_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(StatusCode::FORBIDDEN, "store_id is invalid");
        }
    };

    let row = sqlx::query(
        r#"
        SELECT 1
        FROM store_staff ss
        JOIN store_roles sr ON sr.id = ss.role_id
        JOIN store_role_permissions srp ON srp.role_id = sr.id
        JOIN permissions p ON p.id = srp.permission_id
        WHERE ss.id = $1
          AND ss.store_id = $2
          AND ss.status = 'active'
          AND p.key = $3
        LIMIT 1
        "#,
    )
    .bind(staff_uuid)
    .bind(store_uuid)
    .bind(permission_key)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(_)) => next.run(req).await,
        Ok(None) => error_response(StatusCode::FORBIDDEN, "permission denied"),
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "db error"),
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    let code = match status {
        StatusCode::UNAUTHORIZED => crate::rpc::json::ErrorCode::Unauthenticated,
        StatusCode::FORBIDDEN => crate::rpc::json::ErrorCode::PermissionDenied,
        StatusCode::INTERNAL_SERVER_ERROR => crate::rpc::json::ErrorCode::Internal,
        _ => crate::rpc::json::ErrorCode::PermissionDenied,
    };
    let body = Json(ConnectError {
        code,
        message: message.to_string(),
    });
    (status, body).into_response()
}

#[derive(Clone, Copy, Debug)]
pub enum PermissionKey {
    CatalogRead,
    CatalogWrite,
    OrdersRead,
    OrdersWrite,
    PromotionsWrite,
    AuctionRead,
    AuctionWrite,
    SettingsRead,
    SettingsWrite,
    StaffManage,
    AuditRead,
    CustomersRead,
    CustomersWrite,
}

impl PermissionKey {
    pub fn as_str(self) -> &'static str {
        match self {
            PermissionKey::CatalogRead => "catalog.read",
            PermissionKey::CatalogWrite => "catalog.write",
            PermissionKey::OrdersRead => "orders.read",
            PermissionKey::OrdersWrite => "orders.write",
            PermissionKey::PromotionsWrite => "promotions.write",
            PermissionKey::AuctionRead => "auction.read",
            PermissionKey::AuctionWrite => "auction.write",
            PermissionKey::SettingsRead => "settings.read",
            PermissionKey::SettingsWrite => "settings.write",
            PermissionKey::StaffManage => "staff.manage",
            PermissionKey::AuditRead => "audit.read",
            PermissionKey::CustomersRead => "customers.read",
            PermissionKey::CustomersWrite => "customers.write",
        }
    }
}

pub async fn require_permission_key(
    req: axum::http::Request<Body>,
    next: Next,
    permission: PermissionKey,
) -> Response {
    require_permission(req, next, permission.as_str()).await
}
