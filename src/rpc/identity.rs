use axum::{
    Json,
    body::Bytes,
    http::{HeaderMap, StatusCode},
    extract::State,
    extract::Extension,
};

use crate::{
    AppState,
    pb::pb,
    rpc::json::{ConnectError, parse_request},
    identity,
};

pub async fn sign_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentitySignInResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentitySignInRequest>(&headers, body)?;
    let resp = identity::service::sign_in(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn sign_out(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentitySignOutResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentitySignOutRequest>(&headers, body)?;
    let resp = identity::service::sign_out(&state, req, actor_ctx)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn create_staff(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityCreateStaffResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::IdentityCreateStaffRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::create_staff(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn create_role(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityCreateRoleResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::IdentityCreateRoleRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::create_role(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn assign_role_to_staff(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityAssignRoleResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::IdentityAssignRoleRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::assign_role_to_staff(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn list_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityListRolesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentityListRolesRequest>(&headers, body)?;
    let resp = identity::service::list_roles(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn list_roles_with_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityListRolesWithPermissionsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentityListRolesWithPermissionsRequest>(&headers, body)?;
    let resp = identity::service::list_roles_with_permissions(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn update_role(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityUpdateRoleResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::IdentityUpdateRoleRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::update_role(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn delete_role(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityDeleteRoleResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::IdentityDeleteRoleRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::delete_role(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn list_staff(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityListStaffResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentityListStaffRequest>(&headers, body)?;
    let resp = identity::service::list_staff(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn update_staff(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityUpdateStaffResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::IdentityUpdateStaffRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::update_staff(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn invite_staff(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityInviteStaffResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::IdentityInviteStaffRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::invite_staff(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn transfer_owner(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::IdentityTransferOwnerResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::IdentityTransferOwnerRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::transfer_owner(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}
