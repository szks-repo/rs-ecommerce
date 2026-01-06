use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{
    AppState, cart,
    identity::context::resolve_store_context_without_token_guard,
    infrastructure::search::SearchProduct,
    pages,
    pb::pb,
    product,
    rpc::json::{ConnectError, parse_request, require_tenant_id},
};

pub async fn list_products(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListProductsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListProductsRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant)?;
    let products = product::service::list_products(&state, tenant_id).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListProductsResponse {
            products,
            page: Some(pb::PageResult {
                next_page_token: String::new(),
            }),
        }),
    ))
}

pub async fn get_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::GetProductResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::GetProductRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant)?;
    let product = product::service::get_product(&state, tenant_id, req.product_id).await?;
    Ok((StatusCode::OK, Json(pb::GetProductResponse { product })))
}

pub async fn search_products(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::SearchProductsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::SearchProductsRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant)?;
    let hits = state.search.search_products(&req.query, 50, &tenant_id).await?;
    let products = hits_to_products(hits, tenant_id);
    Ok((
        StatusCode::OK,
        Json(pb::SearchProductsResponse {
            products,
            page: Some(pb::PageResult {
                next_page_token: String::new(),
            }),
        }),
    ))
}

pub async fn get_page_by_slug(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::GetPageBySlugResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::GetPageBySlugRequest>(&headers, body)?;
    let (store_id, tenant_id) = resolve_store_context_without_token_guard(&state, req.store, req.tenant).await?;
    let page = pages::service::get_page_by_slug(&state, store_id, tenant_id, req.slug).await?;
    Ok((StatusCode::OK, Json(pb::GetPageBySlugResponse { page: Some(page) })))
}

fn hits_to_products(hits: Vec<SearchProduct>, tenant_id: String) -> Vec<pb::Product> {
    hits.into_iter()
        .map(|hit| pb::Product {
            id: hit.id,
            vendor_id: hit.vendor_id,
            title: hit.title,
            description: hit.description,
            status: hit.status,
            variants: Vec::new(),
            updated_at: None,
            tax_rule_id: String::new(),
        })
        .filter(|p| !tenant_id.is_empty() && !p.id.is_empty())
        .collect()
}

pub async fn create_cart(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateCartResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateCartRequest>(&headers, body)?;
    let cart = cart::service::create_cart(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(pb::CreateCartResponse { cart: Some(cart) })))
}

pub async fn add_cart_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::AddCartItemResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::AddCartItemRequest>(&headers, body)?;
    let cart = cart::service::add_cart_item(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(pb::AddCartItemResponse { cart: Some(cart) })))
}

pub async fn update_cart_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateCartItemResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateCartItemRequest>(&headers, body)?;
    let cart = cart::service::update_cart_item(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(pb::UpdateCartItemResponse { cart: Some(cart) })))
}

pub async fn remove_cart_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::RemoveCartItemResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::RemoveCartItemRequest>(&headers, body)?;
    let cart = cart::service::remove_cart_item(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(pb::RemoveCartItemResponse { cart: Some(cart) })))
}

pub async fn get_cart(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::GetCartResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::GetCartRequest>(&headers, body)?;
    let cart = cart::service::get_cart(&state, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(pb::GetCartResponse { cart: Some(cart) })))
}

pub async fn checkout(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CheckoutResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CheckoutRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    let order = cart::service::checkout(&state, tenant_id, req)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((StatusCode::OK, Json(pb::CheckoutResponse { order: Some(order) })))
}

pub async fn get_order(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::GetOrderResponse>), (StatusCode, Json<ConnectError>)> {
    let _req = parse_request::<pb::GetOrderRequest>(&headers, body)?;
    Ok((StatusCode::OK, Json(pb::GetOrderResponse { order: None })))
}
