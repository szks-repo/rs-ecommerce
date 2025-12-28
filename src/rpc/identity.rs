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
    rpc::json::{ConnectError, parse_request},
    identity,
};

pub async fn sign_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentitySignInResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentitySignInRequest>(&headers, body)?;
    db::ping(&state).await?;
    let resp = identity::service::sign_in(&state, req).await?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn create_staff(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityCreateStaffResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentityCreateStaffRequest>(&headers, body)?;
    db::ping(&state).await?;
    let resp = identity::service::create_staff(&state, req).await?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn create_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityCreateRoleResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentityCreateRoleRequest>(&headers, body)?;
    db::ping(&state).await?;
    let resp = identity::service::create_role(&state, req).await?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn assign_role_to_staff(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityAssignRoleResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentityAssignRoleRequest>(&headers, body)?;
    db::ping(&state).await?;
    let resp = identity::service::assign_role_to_staff(&state, req).await?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn list_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityListRolesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentityListRolesRequest>(&headers, body)?;
    db::ping(&state).await?;
    let resp = identity::service::list_roles(&state, req).await?;
    Ok((StatusCode::OK, Json(resp)))
}
