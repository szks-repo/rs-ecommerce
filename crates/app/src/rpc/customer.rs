use axum::{
    Json,
    body::Bytes,
    extract::Extension,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{
    AppState, customer,
    identity::context::resolve_store_context,
    pb::pb,
    rpc::json::{ConnectError, parse_request},
};

pub async fn list_customers(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListCustomersResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListCustomersRequest>(&headers, body)?;
    let (store_id, tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let (customers, page) = customer::service::list_customers(&state, store_id, tenant_id, req.query, req.page)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::ListCustomersResponse {
            customers,
            page: Some(page),
        }),
    ))
}

pub async fn get_customer(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::GetCustomerResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::GetCustomerRequest>(&headers, body)?;
    let (store_id, tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let (customer, profile, identities, addresses) =
        customer::service::get_customer(&state, store_id, tenant_id, req.customer_id)
            .await
            .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::GetCustomerResponse {
            customer: Some(customer),
            profile: Some(profile),
            identities,
            addresses,
        }),
    ))
}

pub async fn create_customer(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateCustomerResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateCustomerRequest>(&headers, body)?;
    let (store_id, tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let profile = req.profile.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "profile is required".to_string(),
            }),
        )
    })?;
    let actor = req.actor.or(actor_ctx);
    let (customer, profile, matched_existing) =
        customer::service::create_customer(&state, store_id, tenant_id, profile, req.identities, actor)
            .await
            .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateCustomerResponse {
            customer: Some(customer),
            profile: Some(profile),
            matched_existing,
        }),
    ))
}

pub async fn update_customer(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateCustomerResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateCustomerRequest>(&headers, body)?;
    let (store_id, tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let profile = req.profile.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "profile is required".to_string(),
            }),
        )
    })?;
    let actor = req.actor.or(actor_ctx);
    let (customer, profile) = customer::service::update_customer(
        &state,
        store_id,
        tenant_id,
        req.customer_id,
        profile,
        req.customer_status,
        actor,
    )
    .await
    .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateCustomerResponse {
            customer: Some(customer),
            profile: Some(profile),
        }),
    ))
}

pub async fn upsert_customer_identity(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpsertCustomerIdentityResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpsertCustomerIdentityRequest>(&headers, body)?;
    let (_store_id, tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let identity = req.identity.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "identity is required".to_string(),
            }),
        )
    })?;
    let identity = customer::service::upsert_customer_identity(&state, tenant_id, req.customer_id, identity, actor)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::UpsertCustomerIdentityResponse {
            identity: Some(identity),
        }),
    ))
}

pub async fn upsert_customer_address(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpsertCustomerAddressResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpsertCustomerAddressRequest>(&headers, body)?;
    let (_store_id, _tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let address = req.address.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "address is required".to_string(),
            }),
        )
    })?;
    let address = customer::service::upsert_customer_address(&state, req.customer_id, address, actor)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::UpsertCustomerAddressResponse { address: Some(address) }),
    ))
}

pub async fn list_customer_metafield_definitions(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListCustomerMetafieldDefinitionsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListCustomerMetafieldDefinitionsRequest>(&headers, body)?;
    let (_store_id, _tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let definitions = customer::service::list_customer_metafield_definitions(&state)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::ListCustomerMetafieldDefinitionsResponse { definitions }),
    ))
}

pub async fn create_customer_metafield_definition(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateCustomerMetafieldDefinitionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateCustomerMetafieldDefinitionRequest>(&headers, body)?;
    let (_store_id, _tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let _actor = req.actor.or(actor_ctx);
    let input = req.definition.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "definition is required".to_string(),
            }),
        )
    })?;
    let definition = customer::service::create_customer_metafield_definition(&state, input)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateCustomerMetafieldDefinitionResponse {
            definition: Some(definition),
        }),
    ))
}

pub async fn update_customer_metafield_definition(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateCustomerMetafieldDefinitionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateCustomerMetafieldDefinitionRequest>(&headers, body)?;
    let (_store_id, _tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let _actor = req.actor.or(actor_ctx);
    let input = req.definition.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "definition is required".to_string(),
            }),
        )
    })?;
    let definition = customer::service::update_customer_metafield_definition(&state, req.definition_id, input)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateCustomerMetafieldDefinitionResponse {
            definition: Some(definition),
        }),
    ))
}

pub async fn list_customer_metafield_values(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListCustomerMetafieldValuesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListCustomerMetafieldValuesRequest>(&headers, body)?;
    let (_store_id, tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let values = customer::service::list_customer_metafield_values(&state, tenant_id, req.customer_id)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(pb::ListCustomerMetafieldValuesResponse { values })))
}

pub async fn upsert_customer_metafield_value(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpsertCustomerMetafieldValueResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpsertCustomerMetafieldValueRequest>(&headers, body)?;
    let (_store_id, tenant_id) = resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    customer::service::upsert_customer_metafield_value(
        &state,
        tenant_id,
        req.customer_id,
        req.definition_id,
        req.value_json,
        actor,
    )
    .await
    .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(pb::UpsertCustomerMetafieldValueResponse {})))
}
