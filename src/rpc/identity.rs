use axum::{
    Json,
    body::Bytes,
    extract::Extension,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{
    AppState, identity,
    pb::pb,
    rpc::actor::AuthContext,
    rpc::json::{ConnectError, parse_request},
};

const REFRESH_COOKIE_PREFIX: &str = "refresh_token_";
const REFRESH_TOKEN_MAX_AGE: i64 = 60 * 60 * 24 * 30;

pub async fn sign_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentitySignInRequest>(&headers, body)?;
    let result = identity::service::sign_in_with_refresh(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    let cookie_name = refresh_cookie_name(&result.response.store_id);
    let mut response = Json(result.response).into_response();
    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        build_refresh_cookie(&cookie_name, &result.refresh_token, REFRESH_TOKEN_MAX_AGE),
    );
    Ok(response)
}

pub async fn sign_out(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    Extension(auth_ctx): Extension<Option<AuthContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentitySignOutRequest>(&headers, body)?;
    let store_id_from_auth = auth_ctx.as_ref().and_then(|ctx| ctx.store_id.clone());
    let session_id = auth_ctx.and_then(|ctx| ctx.session_id);
    let store_id = req
        .store
        .as_ref()
        .and_then(|s| {
            if s.store_id.is_empty() {
                None
            } else {
                Some(s.store_id.clone())
            }
        })
        .or_else(|| store_id_from_auth);
    let resp = identity::service::sign_out(&state, req, actor_ctx, session_id)
        .await
        .map_err(|err| err.into_connect())?;
    if let Some(store_id) = store_id {
        let cookie_name = refresh_cookie_name(&store_id);
        let mut response = Json(resp).into_response();
        response.headers_mut().insert(
            axum::http::header::SET_COOKIE,
            clear_refresh_cookie(&cookie_name),
        );
        return Ok(response);
    }
    Ok(Json(resp).into_response())
}

pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::IdentityRefreshTokenRequest>(&headers, body)?;
    let store_id = req
        .store
        .as_ref()
        .and_then(|s| {
            if s.store_id.is_empty() {
                None
            } else {
                Some(s.store_id.clone())
            }
        })
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_id is required".to_string(),
                }),
            )
        })?;
    let cookie_name = refresh_cookie_name(&store_id);
    let refresh_token = extract_cookie(&headers, &cookie_name).unwrap_or_default();
    let result = identity::service::refresh_token(&state, req, refresh_token)
        .await
        .map_err(|err| err.into_connect())?;
    let mut response = Json(result.response).into_response();
    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        build_refresh_cookie(&cookie_name, &result.refresh_token, REFRESH_TOKEN_MAX_AGE),
    );
    Ok(response)
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
) -> Result<
    (
        StatusCode,
        Json<pb::IdentityListRolesWithPermissionsResponse>,
    ),
    (StatusCode, Json<ConnectError>),
> {
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

pub async fn list_staff_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::IdentityListStaffSessionsResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let req = parse_request::<pb::IdentityListStaffSessionsRequest>(&headers, body)?;
    let resp = identity::service::list_staff_sessions(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn force_sign_out_staff(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::IdentityForceSignOutStaffResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let mut req = parse_request::<pb::IdentityForceSignOutStaffRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::force_sign_out_staff(&state, req)
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

fn refresh_cookie_name(store_id: &str) -> String {
    format!("{REFRESH_COOKIE_PREFIX}{store_id}")
}

fn build_refresh_cookie(name: &str, value: &str, max_age_seconds: i64) -> HeaderValue {
    let mut cookie =
        format!("{name}={value}; Max-Age={max_age_seconds}; Path=/; HttpOnly; SameSite=Lax");
    if std::env::var("AUTH_COOKIE_SECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).unwrap()
}

fn clear_refresh_cookie(name: &str) -> HeaderValue {
    let mut cookie = format!("{name}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax");
    if std::env::var("AUTH_COOKIE_SECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).unwrap()
}

fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix(&format!("{name}=")) {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
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
) -> Result<(StatusCode, Json<pb::IdentityTransferOwnerResponse>), (StatusCode, Json<ConnectError>)>
{
    let mut req = parse_request::<pb::IdentityTransferOwnerRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = identity::service::transfer_owner(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(resp)))
}
