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
    setup,
};

pub async fn initialize_store(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::InitializeStoreResponse>), (StatusCode, Json<ConnectError>)> {
    let mut req = parse_request::<pb::InitializeStoreRequest>(&headers, body)?;
    if req.actor.is_none() {
        req.actor = actor_ctx;
    }
    let resp = setup::service::initialize_store(&state, req).await?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn validate_store_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ValidateStoreCodeResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ValidateStoreCodeRequest>(&headers, body)?;
    let resp = setup::service::validate_store_code(&state, req).await?;
    Ok((StatusCode::OK, Json(resp)))
}
