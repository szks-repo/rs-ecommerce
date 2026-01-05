use axum::{
    Json,
    body::Bytes,
    extract::Extension,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{
    AppState, auction,
    pb::pb,
    rpc::json::{ConnectError, parse_request},
};

pub async fn create_auction(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CreateAuctionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CreateAuctionRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let actor = req.actor.clone().or(actor_ctx);
    let auction = auction::service::create_auction(&state, store_id, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::CreateAuctionResponse { auction: Some(auction) }),
    ))
}

pub async fn update_auction(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::UpdateAuctionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::UpdateAuctionRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let actor = req.actor.clone().or(actor_ctx);
    let auction = auction::service::update_auction(&state, store_id, req, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::UpdateAuctionResponse { auction: Some(auction) }),
    ))
}

pub async fn list_auctions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListAuctionsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListAuctionsRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let (auctions, page) = auction::service::list_auctions(&state, store_id, req.status, req.page).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListAuctionsResponse {
            auctions,
            page: Some(page),
        }),
    ))
}

pub async fn get_auction(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::GetAuctionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::GetAuctionRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let auction = auction::service::get_auction(&state, store_id, req.auction_id).await?;
    Ok((StatusCode::OK, Json(pb::GetAuctionResponse { auction: Some(auction) })))
}

pub async fn place_bid(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::PlaceBidResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::PlaceBidRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let actor = req.actor.clone().or(actor_ctx);
    let amount = req.amount.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "amount is required".to_string(),
            }),
        )
    })?;
    let (auction, bid) =
        auction::service::place_bid(&state, store_id, req.auction_id, req.customer_id, amount, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::PlaceBidResponse {
            auction: Some(auction),
            bid: Some(bid),
        }),
    ))
}

pub async fn list_bids(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListBidsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListBidsRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let bids = auction::service::list_bids(&state, store_id, req.auction_id).await?;
    Ok((StatusCode::OK, Json(pb::ListBidsResponse { bids })))
}

pub async fn set_auto_bid(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::SetAutoBidResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::SetAutoBidRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let actor = req.actor.clone().or(actor_ctx);
    let (auction, auto_bid) = auction::service::set_auto_bid(
        &state,
        store_id,
        req.auction_id,
        req.customer_id,
        req.max_amount,
        req.enabled,
        actor,
    )
    .await?;
    Ok((
        StatusCode::OK,
        Json(pb::SetAutoBidResponse {
            auction: Some(auction),
            auto_bid: Some(auto_bid),
        }),
    ))
}

pub async fn list_auto_bids(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListAutoBidsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListAutoBidsRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let auto_bids = auction::service::list_auto_bids(&state, store_id, req.auction_id).await?;
    Ok((StatusCode::OK, Json(pb::ListAutoBidsResponse { auto_bids })))
}

pub async fn close_auction(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::CloseAuctionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::CloseAuctionRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let actor = req.actor.clone().or(actor_ctx);
    let auction = auction::service::close_auction(&state, store_id, req.auction_id, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::CloseAuctionResponse { auction: Some(auction) }),
    ))
}

pub async fn approve_auction(
    State(state): State<AppState>,
    Extension(actor_ctx): Extension<Option<pb::ActorContext>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ApproveAuctionResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ApproveAuctionRequest>(&headers, body)?;
    let store_id = auction::service::resolve_context(&state, req.store.clone()).await?;
    let actor = req.actor.or(actor_ctx);
    let auction = auction::service::approve_auction(&state, store_id, req.auction_id, actor).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ApproveAuctionResponse { auction: Some(auction) }),
    ))
}
