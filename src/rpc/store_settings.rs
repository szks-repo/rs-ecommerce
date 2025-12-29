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
    store_settings,
};

pub async fn get_store_settings(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::GetStoreSettingsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::GetStoreSettingsRequest>(&headers, body)?;
    let (store_id, _tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let settings = store_settings::service::get_store_settings(&state, store_id, _tenant_id).await?;
    Ok((StatusCode::OK, Json(pb::GetStoreSettingsResponse { settings: Some(settings) })))
}

pub async fn update_store_settings(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateStoreSettingsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateStoreSettingsRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let settings = req.settings.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "settings is required".to_string(),
            }),
        )
    })?;
    let settings =
        store_settings::service::update_store_settings(&state, store_id, tenant_id, settings, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpdateStoreSettingsResponse { settings: Some(settings) })))
}

pub async fn initialize_store_settings(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::InitializeStoreSettingsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::InitializeStoreSettingsRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let settings = req.settings.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "settings is required".to_string(),
            }),
        )
    })?;
    let mall = req.mall.unwrap_or(pb::MallSettings {
        enabled: false,
        commission_rate: 0.0,
        vendor_approval_required: true,
    });
    let (settings, mall) =
        store_settings::service::initialize_store_settings(
            &state,
            store_id,
            tenant_id,
            settings,
            mall,
            actor,
        )
        .await?;
    Ok((
        StatusCode::OK,
        Json(pb::InitializeStoreSettingsResponse {
            settings: Some(settings),
            mall: Some(mall),
        }),
    ))
}

pub async fn get_mall_settings(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::GetMallSettingsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::GetMallSettingsRequest>(&headers, body)?;
    let (store_id, _tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let mall = store_settings::service::get_mall_settings(&state, store_id, _tenant_id).await?;
    Ok((StatusCode::OK, Json(pb::GetMallSettingsResponse { mall: Some(mall) })))
}

pub async fn update_mall_settings(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateMallSettingsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateMallSettingsRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let mall = req.mall.unwrap_or(pb::MallSettings {
        enabled: false,
        commission_rate: 0.0,
        vendor_approval_required: true,
    });
    let mall = store_settings::service::update_mall_settings(&state, store_id, tenant_id, mall, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpdateMallSettingsResponse { mall: Some(mall) })))
}

pub async fn list_store_locations(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListStoreLocationsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListStoreLocationsRequest>(&headers, body)?;
    let (store_id, _tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let locations = store_settings::service::list_store_locations(&state, store_id).await?;
    Ok((StatusCode::OK, Json(pb::ListStoreLocationsResponse { locations })))
}

pub async fn upsert_store_location(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpsertStoreLocationResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpsertStoreLocationRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let location = req.location.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "location is required".to_string(),
            }),
        )
    })?;
    let location =
        store_settings::service::upsert_store_location(&state, store_id, tenant_id, location, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpsertStoreLocationResponse { location: Some(location) })))
}

pub async fn delete_store_location(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::DeleteStoreLocationResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::DeleteStoreLocationRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let deleted =
        store_settings::service::delete_store_location(&state, store_id, tenant_id, req.location_id, actor).await?;
    Ok((StatusCode::OK, Json(pb::DeleteStoreLocationResponse { deleted })))
}

pub async fn list_shipping_zones(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListShippingZonesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListShippingZonesRequest>(&headers, body)?;
    let (store_id, _tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let zones = store_settings::service::list_shipping_zones(&state, store_id).await?;
    Ok((StatusCode::OK, Json(pb::ListShippingZonesResponse { zones })))
}

pub async fn upsert_shipping_zone(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpsertShippingZoneResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpsertShippingZoneRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let zone = req.zone.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "zone is required".to_string(),
            }),
        )
    })?;
    let zone =
        store_settings::service::upsert_shipping_zone(&state, store_id, tenant_id, zone, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpsertShippingZoneResponse { zone: Some(zone) })))
}

pub async fn delete_shipping_zone(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::DeleteShippingZoneResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::DeleteShippingZoneRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let deleted =
        store_settings::service::delete_shipping_zone(&state, store_id, tenant_id, req.zone_id, actor).await?;
    Ok((StatusCode::OK, Json(pb::DeleteShippingZoneResponse { deleted })))
}

pub async fn list_shipping_rates(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListShippingRatesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListShippingRatesRequest>(&headers, body)?;
    let (store_id, _tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let rates = store_settings::service::list_shipping_rates(&state, store_id, req.zone_id).await?;
    Ok((StatusCode::OK, Json(pb::ListShippingRatesResponse { rates })))
}

pub async fn upsert_shipping_rate(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpsertShippingRateResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpsertShippingRateRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let rate = req.rate.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "rate is required".to_string(),
            }),
        )
    })?;
    let rate =
        store_settings::service::upsert_shipping_rate(&state, store_id, tenant_id, rate, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpsertShippingRateResponse { rate: Some(rate) })))
}

pub async fn delete_shipping_rate(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::DeleteShippingRateResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::DeleteShippingRateRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let deleted =
        store_settings::service::delete_shipping_rate(&state, store_id, tenant_id, req.rate_id, actor).await?;
    Ok((StatusCode::OK, Json(pb::DeleteShippingRateResponse { deleted })))
}

pub async fn list_tax_rules(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListTaxRulesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListTaxRulesRequest>(&headers, body)?;
    let (store_id, _tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let rules = store_settings::service::list_tax_rules(&state, store_id).await?;
    Ok((StatusCode::OK, Json(pb::ListTaxRulesResponse { rules })))
}

pub async fn upsert_tax_rule(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpsertTaxRuleResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpsertTaxRuleRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let rule = req.rule.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "rule is required".to_string(),
            }),
        )
    })?;
    let rule =
        store_settings::service::upsert_tax_rule(&state, store_id, tenant_id, rule, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpsertTaxRuleResponse { rule: Some(rule) })))
}

pub async fn delete_tax_rule(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::DeleteTaxRuleResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::DeleteTaxRuleRequest>(&headers, body)?;
    let (store_id, tenant_id) =
        store_settings::service::resolve_store_context(&state, req.store, req.tenant).await?;
    let actor = req.actor.or(actor_ctx);
    let deleted =
        store_settings::service::delete_tax_rule(&state, store_id, tenant_id, req.rule_id, actor).await?;
    Ok((StatusCode::OK, Json(pb::DeleteTaxRuleResponse { deleted })))
}
