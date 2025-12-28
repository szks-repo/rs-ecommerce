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
    catalog, order, promotion,
    rpc::json::{ConnectError, parse_request, require_tenant_id},
};

pub async fn list_products(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListProductsAdminResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListProductsAdminRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant)?;
    let products = catalog::service::list_products_admin(&state, tenant_id).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListProductsAdminResponse {
            products,
            page: Some(pb::PageResult {
                next_page_token: String::new(),
            }),
        }),
    ))
}

pub async fn create_product(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateProductResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateProductRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    let product = catalog::service::create_product(&state, tenant_id, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::CreateProductResponse { product: Some(product) })))
}

pub async fn update_product(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateProductResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateProductRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    let product = catalog::service::update_product(&state, tenant_id, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpdateProductResponse { product: Some(product) })))
}

pub async fn create_variant(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateVariantResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateVariantRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let variant = catalog::service::create_variant(&state, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::CreateVariantResponse { variant: Some(variant) })))
}

pub async fn update_variant(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateVariantResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateVariantRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let variant = catalog::service::update_variant(&state, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpdateVariantResponse { variant: Some(variant) })))
}

pub async fn set_inventory(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::SetInventoryResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::SetInventoryRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    let inventory = catalog::service::set_inventory(&state, tenant_id, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::SetInventoryResponse { inventory: Some(inventory) })))
}

pub async fn list_orders(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListOrdersResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListOrdersRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant)?;
    let orders = order::service::list_orders(&state, tenant_id, req.status).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListOrdersResponse {
            orders,
            page: Some(pb::PageResult {
                next_page_token: String::new(),
            }),
        }),
    ))
}

pub async fn update_order_status(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateOrderStatusResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateOrderStatusRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    let order = order::service::update_order_status(&state, tenant_id, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpdateOrderStatusResponse { order: Some(order) })))
}

pub async fn create_shipment(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateShipmentResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateShipmentRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let shipment = order::service::create_shipment(&state, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::CreateShipmentResponse { shipment: Some(shipment) })))
}

pub async fn update_shipment_status(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateShipmentStatusResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateShipmentStatusRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let shipment = order::service::update_shipment_status(&state, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpdateShipmentStatusResponse { shipment: Some(shipment) })))
}

pub async fn create_promotion(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreatePromotionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreatePromotionRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    let promotion = promotion::service::create_promotion(&state, tenant_id, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::CreatePromotionResponse { promotion: Some(promotion) })))
}

pub async fn update_promotion(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdatePromotionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdatePromotionRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    let promotion = promotion::service::update_promotion(&state, tenant_id, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::UpdatePromotionResponse { promotion: Some(promotion) })))
}
