use axum::{
    Json,
    body::Bytes,
    extract::Extension,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{
    AppState, order,
    pb::pb,
    product, promotion,
    rpc::json::{ConnectError, parse_request, require_tenant_id},
};

pub async fn list_products(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListProductsAdminResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListProductsAdminRequest>(&headers, body)?;
    let products = product::service::list_products_admin(&state, req.tenant, req.store).await?;
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

pub async fn list_variants(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListVariantsAdminResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListVariantsAdminRequest>(&headers, body)?;
    let (variants, variant_axes) =
        product::service::list_variants_admin(&state, req.tenant, req.store, req.product_id)
            .await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListVariantsAdminResponse {
            variants,
            page: Some(pb::PageResult {
                next_page_token: String::new(),
            }),
            variant_axes,
        }),
    ))
}

pub async fn list_skus(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListSkusAdminResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListSkusAdminRequest>(&headers, body)?;
    let skus = product::service::list_skus_admin(&state, req.tenant, req.store, req.query).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListSkusAdminResponse {
            skus,
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
    let product = product::service::create_product(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateProductResponse {
            product: Some(product),
        }),
    ))
}

pub async fn update_product(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateProductResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateProductRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let product = product::service::update_product(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateProductResponse {
            product: Some(product),
        }),
    ))
}

pub async fn list_categories(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListCategoriesAdminResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListCategoriesAdminRequest>(&headers, body)?;
    let categories = product::service::list_categories_admin(&state, req.store, req.status).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListCategoriesAdminResponse { categories }),
    ))
}

pub async fn create_category(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateCategoryResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateCategoryRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let category = product::service::create_category(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateCategoryResponse {
            category: Some(category),
        }),
    ))
}

pub async fn update_category(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateCategoryResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateCategoryRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let category = product::service::update_category(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateCategoryResponse {
            category: Some(category),
        }),
    ))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::DeleteCategoryResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::DeleteCategoryRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let deleted = product::service::delete_category(&state, req, actor).await?;
    Ok((StatusCode::OK, Json(pb::DeleteCategoryResponse { deleted })))
}

pub async fn reorder_categories(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ReorderCategoriesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ReorderCategoriesRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let categories = product::service::reorder_categories(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ReorderCategoriesResponse { categories }),
    ))
}

pub async fn list_category_products(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::ListCategoryProductsResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let req = parse_request::<pb::ListCategoryProductsRequest>(&headers, body)?;
    let products =
        product::service::list_category_products_admin(&state, req.store, req.category_id).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListCategoryProductsResponse { products }),
    ))
}

pub async fn reorder_category_products(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::ReorderCategoryProductsResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let req = parse_request::<pb::ReorderCategoryProductsRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let products = product::service::reorder_category_products(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ReorderCategoryProductsResponse { products }),
    ))
}

pub async fn create_variant(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateVariantResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateVariantRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let variant = product::service::create_variant(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateVariantResponse {
            variant: Some(variant),
        }),
    ))
}

pub async fn update_variant(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateVariantResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateVariantRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let variant = product::service::update_variant(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateVariantResponse {
            variant: Some(variant),
        }),
    ))
}

pub async fn list_media_assets(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListMediaAssetsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListMediaAssetsRequest>(&headers, body)?;
    let assets =
        product::media::list_media_assets(&state, req.store, req.tenant, req.query).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListMediaAssetsResponse {
            assets,
            page: Some(pb::PageResult {
                next_page_token: String::new(),
            }),
        }),
    ))
}

pub async fn create_media_asset(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateMediaAssetResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateMediaAssetRequest>(&headers, body)?;
    let _actor = req.actor.clone().or(actor_ctx);
    let asset = product::media::create_media_asset(
        &state,
        req.store,
        req.tenant,
        req.asset.unwrap_or_default(),
    )
    .await?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateMediaAssetResponse { asset: Some(asset) }),
    ))
}

pub async fn create_media_upload_url(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateMediaUploadUrlResponse>), (StatusCode, Json<ConnectError>)>
{
    let req = parse_request::<pb::CreateMediaUploadUrlRequest>(&headers, body)?;
    let resp = product::media::create_media_upload_url(
        &state,
        req.store,
        req.tenant,
        req.filename,
        req.content_type,
        req.size_bytes,
    )
    .await?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn list_sku_images(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListSkuImagesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListSkuImagesRequest>(&headers, body)?;
    let images = product::media::list_sku_images(&state, req.store, req.tenant, req.sku_id).await?;
    Ok((StatusCode::OK, Json(pb::ListSkuImagesResponse { images })))
}

pub async fn set_sku_images(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::SetSkuImagesResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::SetSkuImagesRequest>(&headers, body)?;
    let _actor = req.actor.clone().or(actor_ctx);
    let images =
        product::media::set_sku_images(&state, req.store, req.tenant, req.sku_id, req.images)
            .await?;
    Ok((StatusCode::OK, Json(pb::SetSkuImagesResponse { images })))
}

pub async fn list_digital_assets(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListDigitalAssetsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListDigitalAssetsRequest>(&headers, body)?;
    let assets =
        product::digital::list_digital_assets(&state, req.store, req.tenant, req.sku_id).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListDigitalAssetsResponse { assets }),
    ))
}

pub async fn create_digital_asset(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateDigitalAssetResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateDigitalAssetRequest>(&headers, body)?;
    let _actor = req.actor.clone().or(actor_ctx);
    let asset = product::digital::create_digital_asset(
        &state,
        req.store,
        req.tenant,
        req.sku_id,
        req.asset.unwrap_or_default(),
    )
    .await?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateDigitalAssetResponse { asset: Some(asset) }),
    ))
}

pub async fn create_digital_upload_url(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateDigitalUploadUrlResponse>), (StatusCode, Json<ConnectError>)>
{
    let req = parse_request::<pb::CreateDigitalUploadUrlRequest>(&headers, body)?;
    let resp = product::digital::create_digital_upload_url(
        &state,
        req.store,
        req.tenant,
        req.sku_id,
        req.filename,
        req.content_type,
        req.size_bytes,
    )
    .await?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn create_digital_download_url(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::CreateDigitalDownloadUrlResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let req = parse_request::<pb::CreateDigitalDownloadUrlRequest>(&headers, body)?;
    let resp =
        product::digital::create_digital_download_url(&state, req.store, req.tenant, req.asset_id)
            .await?;
    Ok((StatusCode::OK, Json(resp)))
}

pub async fn set_inventory(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::SetInventoryResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::SetInventoryRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let inventory = product::service::set_inventory(&state, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::SetInventoryResponse {
            inventory: Some(inventory),
        }),
    ))
}

pub async fn list_orders(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListOrdersResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListOrdersRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant)?;
    let orders = order::service::list_orders(&state, tenant_id, req.status)
        .await
        .map_err(|err| err.into_connect())?;
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
    let order = order::service::update_order_status(&state, tenant_id, req, actor)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateOrderStatusResponse { order: Some(order) }),
    ))
}

pub async fn create_shipment(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateShipmentResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateShipmentRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let shipment = order::service::create_shipment(&state, req, actor)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateShipmentResponse {
            shipment: Some(shipment),
        }),
    ))
}

pub async fn update_shipment_status(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateShipmentStatusResponse>), (StatusCode, Json<ConnectError>)>
{
    let req = parse_request::<pb::UpdateShipmentStatusRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    let shipment = order::service::update_shipment_status(&state, req, actor)
        .await
        .map_err(|err| err.into_connect())?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateShipmentStatusResponse {
            shipment: Some(shipment),
        }),
    ))
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
    Ok((
        StatusCode::OK,
        Json(pb::CreatePromotionResponse {
            promotion: Some(promotion),
        }),
    ))
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
    Ok((
        StatusCode::OK,
        Json(pb::UpdatePromotionResponse {
            promotion: Some(promotion),
        }),
    ))
}

pub async fn list_product_metafield_definitions(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::ListProductMetafieldDefinitionsResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let _req = parse_request::<pb::ListProductMetafieldDefinitionsRequest>(&headers, body)?;
    let definitions = product::service::list_product_metafield_definitions(&state).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListProductMetafieldDefinitionsResponse { definitions }),
    ))
}

pub async fn create_product_metafield_definition(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::CreateProductMetafieldDefinitionResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let req = parse_request::<pb::CreateProductMetafieldDefinitionRequest>(&headers, body)?;
    let _actor = req.actor.clone().or(actor_ctx);
    let definition = product::service::create_product_metafield_definition(
        &state,
        req.definition.unwrap_or_default(),
    )
    .await?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateProductMetafieldDefinitionResponse {
            definition: Some(definition),
        }),
    ))
}

pub async fn update_product_metafield_definition(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::UpdateProductMetafieldDefinitionResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let req = parse_request::<pb::UpdateProductMetafieldDefinitionRequest>(&headers, body)?;
    let _actor = req.actor.clone().or(actor_ctx);
    let definition = product::service::update_product_metafield_definition(
        &state,
        req.definition_id,
        req.definition.unwrap_or_default(),
    )
    .await?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateProductMetafieldDefinitionResponse {
            definition: Some(definition),
        }),
    ))
}

pub async fn list_product_metafield_values(
    State(state): State<AppState>,
    Extension(_actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::ListProductMetafieldValuesResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let req = parse_request::<pb::ListProductMetafieldValuesRequest>(&headers, body)?;
    let values = product::service::list_product_metafield_values(
        &state,
        req.store,
        req.product_id,
    )
    .await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListProductMetafieldValuesResponse { values }),
    ))
}

pub async fn upsert_product_metafield_value(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<
    (StatusCode, Json<pb::UpsertProductMetafieldValueResponse>),
    (StatusCode, Json<ConnectError>),
> {
    let req = parse_request::<pb::UpsertProductMetafieldValueRequest>(&headers, body)?;
    let actor = req.actor.clone().or(actor_ctx);
    product::service::upsert_product_metafield_value(
        &state,
        req.store,
        req.product_id,
        req.definition_id,
        req.value_json,
        actor,
    )
    .await?;
    Ok((StatusCode::OK, Json(pb::UpsertProductMetafieldValueResponse {})))
}
