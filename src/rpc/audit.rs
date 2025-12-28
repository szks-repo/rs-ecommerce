use axum::{
    Json,
    body::Bytes,
    http::{HeaderMap, StatusCode},
    extract::State,
};

use crate::{
    AppState,
    pb::pb,
    infrastructure::db,
    rpc::json::{ConnectError, parse_request, require_tenant_id},
    audit,
};

pub async fn list_audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListAuditLogsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListAuditLogsRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    db::ping(&state).await?;
    let (logs, page) = audit::service::list_audit_logs(&state, tenant_id, req).await?;
    Ok((StatusCode::OK, Json(pb::ListAuditLogsResponse { logs, page: Some(page) })))
}
