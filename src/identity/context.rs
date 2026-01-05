use axum::{Json, http::StatusCode};
use sqlx::Row;

use crate::{
    AppState,
    infrastructure::db,
    pb::pb,
    rpc::json::ConnectError,
    rpc::request_context,
    shared::ids::{StoreId, TenantId},
};

pub async fn resolve_store_context(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> Result<(String, String), (StatusCode, Json<ConnectError>)> {
    if let Some(ctx) = request_context::current() {
        if let Some(auth_store) = ctx.store_id.as_deref()
            && let Some(store_id) = store.as_ref().and_then(|s| {
                if s.store_id.is_empty() {
                    None
                } else {
                    Some(s.store_id.as_str())
                }
            })
            && store_id != auth_store
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "store_id does not match token".to_string(),
                }),
            ));
        }
        if let Some(auth_tenant) = ctx.tenant_id.as_deref()
            && let Some(tenant_id) = tenant.as_ref().and_then(|t| {
                if t.tenant_id.is_empty() {
                    None
                } else {
                    Some(t.tenant_id.as_str())
                }
            })
            && tenant_id != auth_tenant
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "tenant_id does not match token".to_string(),
                }),
            ));
        }
    }

    if let Some(store_id) = store.as_ref().and_then(|s| {
        if s.store_id.is_empty() {
            None
        } else {
            Some(s.store_id.as_str())
        }
    }) {
        let store_uuid = StoreId::parse(store_id)?;
        let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
            .bind(store_uuid.as_uuid())
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_id not found".to_string(),
                }),
            ));
        };
        let tenant_id: String = row.get("tenant_id");
        return Ok((store_id.to_string(), tenant_id));
    }
    if let Some(store_code) = store.as_ref().and_then(|s| {
        if s.store_code.is_empty() {
            None
        } else {
            Some(s.store_code.as_str())
        }
    }) {
        let row = sqlx::query("SELECT id::text as id, tenant_id::text as tenant_id FROM stores WHERE code = $1")
            .bind(store_code)
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_code not found".to_string(),
                }),
            ));
        };
        let store_id: String = row.get("id");
        let tenant_id: String = row.get("tenant_id");
        if let Some(ctx) = request_context::current()
            && let Some(auth_store) = ctx.store_id
            && auth_store != store_id
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "store_code does not match token".to_string(),
                }),
            ));
        }
        return Ok((store_id, tenant_id));
    }
    if let Some(ctx) = request_context::current()
        && let (Some(store_id), Some(tenant_id)) = (ctx.store_id, ctx.tenant_id)
    {
        return Ok((store_id, tenant_id));
    }
    if let Some(tenant_id) = tenant.and_then(|t| {
        if t.tenant_id.is_empty() {
            None
        } else {
            Some(t.tenant_id)
        }
    }) {
        let tenant_uuid = TenantId::parse(&tenant_id)?;
        let row = sqlx::query("SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1")
            .bind(tenant_uuid.as_uuid())
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "tenant_id not found".to_string(),
                }),
            ));
        };
        let store_id: String = row.get("id");
        return Ok((store_id, tenant_id));
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ConnectError {
            code: crate::rpc::json::ErrorCode::InvalidArgument,
            message: "store.store_id or tenant.tenant_id is required".to_string(),
        }),
    ))
}

pub async fn resolve_store_context_without_token_guard(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> Result<(String, String), (StatusCode, Json<ConnectError>)> {
    if let Some(store_id) = store.as_ref().and_then(|s| {
        if s.store_id.is_empty() {
            None
        } else {
            Some(s.store_id.as_str())
        }
    }) {
        let store_uuid = StoreId::parse(store_id)?;
        let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
            .bind(store_uuid.as_uuid())
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_id not found".to_string(),
                }),
            ));
        };
        let tenant_id: String = row.get("tenant_id");
        return Ok((store_id.to_string(), tenant_id));
    }
    if let Some(store_code) = store.as_ref().and_then(|s| {
        if s.store_code.is_empty() {
            None
        } else {
            Some(s.store_code.as_str())
        }
    }) {
        let row = sqlx::query("SELECT id::text as id, tenant_id::text as tenant_id FROM stores WHERE code = $1")
            .bind(store_code)
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_code not found".to_string(),
                }),
            ));
        };
        let store_id: String = row.get("id");
        let tenant_id: String = row.get("tenant_id");
        return Ok((store_id, tenant_id));
    }
    if let Some(tenant_id) = tenant.and_then(|t| {
        if t.tenant_id.is_empty() {
            None
        } else {
            Some(t.tenant_id)
        }
    }) {
        let tenant_uuid = TenantId::parse(&tenant_id)?;
        let row = sqlx::query("SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1")
            .bind(tenant_uuid.as_uuid())
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "tenant_id not found".to_string(),
                }),
            ));
        };
        let store_id: String = row.get("id");
        return Ok((store_id, tenant_id));
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ConnectError {
            code: crate::rpc::json::ErrorCode::InvalidArgument,
            message: "store.store_id or tenant.tenant_id is required".to_string(),
        }),
    ))
}

pub fn parse_uuid(value: &str, field: &str) -> Result<uuid::Uuid, (StatusCode, Json<ConnectError>)> {
    uuid::Uuid::parse_str(value).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: format!("{} is invalid", field),
            }),
        )
    })
}
