use axum::{Json, http::StatusCode};
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AppState,
    infrastructure::audit,
    pb::pb,
    rpc::json::ConnectError,
    shared::{
        audit_action::AuctionAuditAction,
        audit_helpers::{audit_input, to_json_opt},
        ids::{StoreId, parse_uuid},
        money::{money_from_parts, money_to_parts, money_to_parts_opt},
        time::chrono_to_timestamp,
    },
    store_settings::service::resolve_store_context,
};

const AUCTION_TYPE_OPEN: &str = "open";
const AUCTION_TYPE_SEALED: &str = "sealed";

const STATUS_DRAFT: &str = "draft";
const STATUS_SCHEDULED: &str = "scheduled";
const STATUS_RUNNING: &str = "running";
const STATUS_ENDED: &str = "ended";
const STATUS_AWAITING_APPROVAL: &str = "awaiting_approval";
const STATUS_APPROVED: &str = "approved";

const AUTO_BID_STATUS_ACTIVE: &str = "active";
const AUTO_BID_STATUS_DISABLED: &str = "disabled";

#[derive(Debug, Clone)]
struct AutoBidCandidate {
    id: uuid::Uuid,
    customer_id: uuid::Uuid,
    max_amount: i64,
    created_at: DateTime<Utc>,
}

pub async fn create_auction(
    state: &AppState,
    store_id: String,
    req: pb::CreateAuctionRequest,
    actor: Option<pb::ActorContext>,
) -> Result<pb::Auction, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;

    if req.sku_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "sku_id is required".to_string(),
            }),
        ));
    }

    if req.auction_type != AUCTION_TYPE_OPEN && req.auction_type != AUCTION_TYPE_SEALED {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "auction_type is invalid".to_string(),
            }),
        ));
    }
    if req.title.trim().is_empty() {
        return Err(invalid_arg("title is required"));
    }

    let start_at = crate::shared::time::timestamp_to_chrono(req.start_at)
        .ok_or_else(|| invalid_arg("start_at is required"))?;
    let end_at = crate::shared::time::timestamp_to_chrono(req.end_at)
        .ok_or_else(|| invalid_arg("end_at is required"))?;
    if end_at <= start_at {
        return Err(invalid_arg("end_at must be after start_at"));
    }

    let (start_price_amount, start_price_currency) = money_to_parts(req.start_price)?;
    let (reserve_amount, reserve_currency) = money_to_parts_opt(req.reserve_price)?;
    let (buyout_amount, buyout_currency) = money_to_parts_opt(req.buyout_price)?;

    let (increment_amount, increment_currency) = match req.bid_increment {
        Some(_) => money_to_parts(req.bid_increment)?,
        None => {
            return Err(invalid_arg("bid_increment is required"));
        }
    };

    let now = Utc::now();
    let status = if req.status == STATUS_DRAFT {
        STATUS_DRAFT.to_string()
    } else if start_at > now {
        STATUS_SCHEDULED.to_string()
    } else {
        STATUS_RUNNING.to_string()
    };

    let mut tx = state.db.begin().await.map_err(db_error)?;
    let auction_id = uuid::Uuid::new_v4();
    let sku_uuid = parse_uuid(&req.sku_id, "sku_id")?;

    let product_id = sqlx::query(
        r#"
        SELECT v.product_id
        FROM product_skus v
        JOIN products p ON p.id = v.product_id
        WHERE v.id = $1 AND p.store_id = $2
        "#,
    )
    .bind(sku_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(tx.as_mut())
    .await
    .map_err(db_error)?
    .map(|row| row.get::<uuid::Uuid, _>("product_id"));

    let Some(product_id) = product_id else {
        return Err(invalid_arg("sku_id not found"));
    };

    let current_price_amount = if req.auction_type == AUCTION_TYPE_OPEN {
        Some(start_price_amount)
    } else {
        None
    };
    let current_price_currency = if req.auction_type == AUCTION_TYPE_OPEN {
        Some(start_price_currency.clone())
    } else {
        None
    };

    sqlx::query(
        r#"
        INSERT INTO auctions
            (id, store_id, product_id, sku_id, auction_type, status,
             start_at, end_at, bid_increment_amount, bid_increment_currency,
             start_price_amount, start_price_currency,
             reserve_price_amount, reserve_price_currency,
             buyout_price_amount, buyout_price_currency,
             current_price_amount, current_price_currency,
             title, description)
        VALUES
            ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
        "#,
    )
    .bind(auction_id)
    .bind(store_uuid.as_uuid())
    .bind(product_id)
    .bind(sku_uuid)
    .bind(req.auction_type.as_str())
    .bind(status.as_str())
    .bind(start_at)
    .bind(end_at)
    .bind(increment_amount)
    .bind(increment_currency.as_str())
    .bind(start_price_amount)
    .bind(start_price_currency.as_str())
    .bind(reserve_amount)
    .bind(reserve_currency.as_deref())
    .bind(buyout_amount)
    .bind(buyout_currency.as_deref())
    .bind(current_price_amount)
    .bind(current_price_currency.as_deref())
    .bind(req.title.trim())
    .bind(req.description.trim())
    .execute(tx.as_mut())
    .await
    .map_err(db_error)?;

    let auction = pb::Auction {
        id: auction_id.to_string(),
        store_id: store_id.clone(),
        product_id: product_id.to_string(),
        sku_id: req.sku_id,
        auction_type: req.auction_type,
        status,
        start_at: chrono_to_timestamp(Some(start_at)),
        end_at: chrono_to_timestamp(Some(end_at)),
        bid_increment: Some(money_from_parts(
            increment_amount,
            increment_currency.clone(),
        )),
        start_price: Some(money_from_parts(
            start_price_amount,
            start_price_currency.clone(),
        )),
        reserve_price: reserve_amount
            .zip(reserve_currency.clone())
            .map(|(amt, cur)| money_from_parts(amt, cur)),
        buyout_price: buyout_amount
            .zip(buyout_currency.clone())
            .map(|(amt, cur)| money_from_parts(amt, cur)),
        current_price: current_price_amount
            .zip(current_price_currency.clone())
            .map(|(amt, cur)| money_from_parts(amt, cur)),
        winning_price: None,
        winning_bid_id: String::new(),
        approved_by: String::new(),
        approved_at: None,
        created_at: chrono_to_timestamp(Some(now)),
        updated_at: chrono_to_timestamp(Some(now)),
        title: req.title,
        description: req.description,
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            AuctionAuditAction::Create.into(),
            Some("auction"),
            Some(auction.id.clone()),
            None,
            to_json_opt(Some(auction.clone())),
            actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db_error)?;

    Ok(auction)
}

pub async fn update_auction(
    state: &AppState,
    store_id: String,
    req: pb::UpdateAuctionRequest,
    actor: Option<pb::ActorContext>,
) -> Result<pb::Auction, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let auction_uuid = parse_uuid(&req.auction_id, "auction_id")?;

    if req.sku_id.is_empty() {
        return Err(invalid_arg("sku_id is required"));
    }
    if req.auction_type != AUCTION_TYPE_OPEN && req.auction_type != AUCTION_TYPE_SEALED {
        return Err(invalid_arg("auction_type is invalid"));
    }
    if req.title.trim().is_empty() {
        return Err(invalid_arg("title is required"));
    }

    let start_at = crate::shared::time::timestamp_to_chrono(req.start_at)
        .ok_or_else(|| invalid_arg("start_at is required"))?;
    let end_at = crate::shared::time::timestamp_to_chrono(req.end_at)
        .ok_or_else(|| invalid_arg("end_at is required"))?;
    if end_at <= start_at {
        return Err(invalid_arg("end_at must be after start_at"));
    }

    let (start_price_amount, start_price_currency) = money_to_parts(req.start_price)?;
    let (reserve_amount, reserve_currency) = money_to_parts_opt(req.reserve_price)?;
    let (buyout_amount, buyout_currency) = money_to_parts_opt(req.buyout_price)?;
    let (increment_amount, increment_currency) = match req.bid_increment {
        Some(_) => money_to_parts(req.bid_increment)?,
        None => {
            return Err(invalid_arg("bid_increment is required"));
        }
    };

    let mut tx = state.db.begin().await.map_err(db_error)?;
    let row = sqlx::query(
        r#"
        SELECT status
        FROM auctions
        WHERE id = $1 AND store_id = $2
        FOR UPDATE
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_one(tx.as_mut())
    .await
    .map_err(db_error)?;
    let current_status: String = row.get("status");
    if current_status != STATUS_DRAFT {
        return Err(invalid_arg("only draft auctions can be edited"));
    }

    let sku_uuid = parse_uuid(&req.sku_id, "sku_id")?;
    let product_id = sqlx::query(
        r#"
        SELECT v.product_id
        FROM product_skus v
        JOIN products p ON p.id = v.product_id
        WHERE v.id = $1 AND p.store_id = $2
        "#,
    )
    .bind(sku_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(tx.as_mut())
    .await
    .map_err(db_error)?
    .map(|row| row.get::<uuid::Uuid, _>("product_id"));

    let Some(product_id) = product_id else {
        return Err(invalid_arg("sku_id not found"));
    };

    let now = Utc::now();
    let requested_status = req.status.trim();
    let status = if requested_status.is_empty() || requested_status == STATUS_DRAFT {
        STATUS_DRAFT.to_string()
    } else if requested_status == STATUS_SCHEDULED {
        if start_at > now {
            STATUS_SCHEDULED.to_string()
        } else {
            STATUS_RUNNING.to_string()
        }
    } else {
        return Err(invalid_arg("status is invalid"));
    };

    let current_price_amount = if req.auction_type == AUCTION_TYPE_OPEN {
        Some(start_price_amount)
    } else {
        None
    };
    let current_price_currency = if req.auction_type == AUCTION_TYPE_OPEN {
        Some(start_price_currency.clone())
    } else {
        None
    };

    sqlx::query(
        r#"
        UPDATE auctions
        SET sku_id = $1,
            product_id = $2,
            auction_type = $3,
            status = $4,
            start_at = $5,
            end_at = $6,
            bid_increment_amount = $7,
            bid_increment_currency = $8,
            start_price_amount = $9,
            start_price_currency = $10,
            reserve_price_amount = $11,
            reserve_price_currency = $12,
            buyout_price_amount = $13,
            buyout_price_currency = $14,
            current_price_amount = $15,
            current_price_currency = $16,
            title = $17,
            description = $18,
            updated_at = now()
        WHERE id = $19
        "#,
    )
    .bind(sku_uuid)
    .bind(product_id)
    .bind(req.auction_type.as_str())
    .bind(status.as_str())
    .bind(start_at)
    .bind(end_at)
    .bind(increment_amount)
    .bind(increment_currency.as_str())
    .bind(start_price_amount)
    .bind(start_price_currency.as_str())
    .bind(reserve_amount)
    .bind(reserve_currency.as_deref())
    .bind(buyout_amount)
    .bind(buyout_currency.as_deref())
    .bind(current_price_amount)
    .bind(current_price_currency.as_deref())
    .bind(req.title.trim())
    .bind(req.description.trim())
    .bind(auction_uuid)
    .execute(tx.as_mut())
    .await
    .map_err(db_error)?;

    let updated_row = sqlx::query("SELECT * FROM auctions WHERE id = $1")
        .bind(auction_uuid)
        .fetch_one(tx.as_mut())
        .await
        .map_err(db_error)?;
    let auction = auction_from_row(&updated_row);

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            AuctionAuditAction::Update.into(),
            Some("auction"),
            Some(req.auction_id),
            None,
            to_json_opt(Some(auction.clone())),
            actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db_error)?;

    Ok(auction)
}

pub async fn list_auctions(
    state: &AppState,
    store_id: String,
    status: String,
    page: Option<pb::PageInfo>,
) -> Result<(Vec<pb::Auction>, pb::PageResult), (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let (limit, offset) = page_params(page);
    let rows = if status.is_empty() {
        sqlx::query(
            r#"
            SELECT *
            FROM auctions
            WHERE store_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(store_uuid.as_uuid())
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(db_error)?
    } else {
        sqlx::query(
            r#"
            SELECT *
            FROM auctions
            WHERE store_id = $1 AND status = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(store_uuid.as_uuid())
        .bind(status.as_str())
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(db_error)?
    };

    let auctions: Vec<pb::Auction> = rows.into_iter().map(|row| auction_from_row(&row)).collect();
    let mut next_page_token = String::new();
    if (auctions.len() as i64) == limit {
        next_page_token = (offset + limit).to_string();
    }
    Ok((auctions, pb::PageResult { next_page_token }))
}

fn page_params(page: Option<pb::PageInfo>) -> (i64, i64) {
    let page = page.unwrap_or(pb::PageInfo {
        page_size: 50,
        page_token: String::new(),
    });
    let limit = (page.page_size.max(1).min(200)) as i64;
    let offset = page.page_token.parse::<i64>().unwrap_or(0).max(0);
    (limit, offset)
}

pub async fn get_auction(
    state: &AppState,
    store_id: String,
    auction_id: String,
) -> Result<pb::Auction, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let auction_uuid = parse_uuid(&auction_id, "auction_id")?;
    let row = sqlx::query(
        r#"
        SELECT *
        FROM auctions
        WHERE id = $1 AND store_id = $2
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;

    Ok(auction_from_row(&row))
}

pub async fn list_bids(
    state: &AppState,
    store_id: String,
    auction_id: String,
) -> Result<Vec<pb::AuctionBid>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let auction_uuid = parse_uuid(&auction_id, "auction_id")?;

    let rows = sqlx::query(
        r#"
        SELECT id::text as id, auction_id::text as auction_id, customer_id::text as customer_id,
               amount, currency, created_at
        FROM auction_bids
        WHERE auction_id = $1 AND store_id = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::AuctionBid {
            id: row.get("id"),
            auction_id: row.get("auction_id"),
            customer_id: row
                .get::<Option<String>, _>("customer_id")
                .unwrap_or_default(),
            amount: Some(money_from_parts(
                row.get::<i64, _>("amount"),
                row.get::<String, _>("currency"),
            )),
            created_at: chrono_to_timestamp(Some(
                row.get::<chrono::DateTime<Utc>, _>("created_at"),
            )),
        })
        .collect())
}

pub async fn place_bid(
    state: &AppState,
    store_id: String,
    auction_id: String,
    customer_id: String,
    amount: pb::Money,
    actor: Option<pb::ActorContext>,
) -> Result<(pb::Auction, pb::AuctionBid), (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let auction_uuid = parse_uuid(&auction_id, "auction_id")?;
    let (bid_amount, bid_currency) = money_to_parts(Some(amount))?;

    let mut tx = state.db.begin().await.map_err(db_error)?;
    if customer_id.trim().is_empty() {
        return Err(permission_denied("customer_id is required"));
    }
    let customer_uuid = parse_uuid(&customer_id, "customer_id")?;
    let eligible = sqlx::query(
        r#"
        SELECT 1
        FROM customer_profiles cp
        JOIN customers c ON c.id = cp.customer_id
        WHERE cp.store_id = $1
          AND cp.customer_id = $2
          AND cp.status = 'active'
          AND c.status = 'active'
        LIMIT 1
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(customer_uuid)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(db_error)?;
    if eligible.is_none() {
        return Err(permission_denied("customer is not eligible to bid"));
    }
    let row = sqlx::query(
        r#"
        SELECT *
        FROM auctions
        WHERE id = $1 AND store_id = $2
        FOR UPDATE
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_one(tx.as_mut())
    .await
    .map_err(db_error)?;

    let auction_type: String = row.get("auction_type");
    let status: String = row.get("status");
    let start_at: DateTime<Utc> = row.get("start_at");
    let end_at: DateTime<Utc> = row.get("end_at");
    let now = Utc::now();

    if now < start_at {
        return Err(invalid_arg("auction has not started"));
    }
    if now >= end_at {
        return Err(invalid_arg("auction has already ended"));
    }
    if status != STATUS_RUNNING && status != STATUS_SCHEDULED {
        return Err(invalid_arg("auction is not running"));
    }

    let start_price_amount: i64 = row.get("start_price_amount");
    let start_price_currency: String = row.get("start_price_currency");
    if start_price_currency != bid_currency {
        return Err(invalid_arg("currency mismatch"));
    }

    if auction_type == AUCTION_TYPE_OPEN {
        let increment_amount: i64 = row.get("bid_increment_amount");
        let current_price_amount: Option<i64> = row.get("current_price_amount");
        let min_amount = match current_price_amount {
            Some(current) => current + increment_amount,
            None => start_price_amount,
        };
        if bid_amount < min_amount {
            return Err(invalid_arg("bid amount is too low"));
        }
    } else if bid_amount < start_price_amount {
        return Err(invalid_arg("bid amount is too low"));
    }

    let bid_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO auction_bids
            (id, auction_id, store_id, customer_id, amount, currency)
        VALUES ($1,$2,$3,$4,$5,$6)
        "#,
    )
    .bind(bid_id)
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .bind(Some(customer_uuid))
    .bind(bid_amount)
    .bind(bid_currency.as_str())
    .execute(tx.as_mut())
    .await
    .map_err(db_error)?;

    let buyout_amount: Option<i64> = row.get("buyout_price_amount");
    let buyout_currency: Option<String> = row.get("buyout_price_currency");

    let mut next_status = status.clone();
    let mut winning_bid_id: Option<uuid::Uuid> = row.get("winning_bid_id");
    let mut winning_amount: Option<i64> = row.get("winning_price_amount");
    let mut current_bid_id: Option<uuid::Uuid> = row.get("current_bid_id");
    let mut current_price_amount: Option<i64> = row.get("current_price_amount");

    if auction_type == AUCTION_TYPE_OPEN {
        current_bid_id = Some(bid_id);
        current_price_amount = Some(bid_amount);
        winning_bid_id = Some(bid_id);
        winning_amount = Some(bid_amount);
    } else if winning_amount.map(|amt| bid_amount > amt).unwrap_or(true) {
        winning_amount = Some(bid_amount);
        winning_bid_id = Some(bid_id);
    }

    if let (Some(buyout), Some(cur)) = (buyout_amount, buyout_currency.clone())
        && cur == bid_currency && bid_amount >= buyout {
            next_status = STATUS_AWAITING_APPROVAL.to_string();
        }

    if status == STATUS_SCHEDULED && now >= start_at {
        next_status = STATUS_RUNNING.to_string();
    }

    sqlx::query(
        r#"
        UPDATE auctions
        SET status = $1,
            current_bid_id = $2,
            current_price_amount = $3,
            current_price_currency = $4,
            winning_bid_id = $5,
            winning_price_amount = $6,
            winning_price_currency = $7,
            updated_at = now()
        WHERE id = $8
        "#,
    )
    .bind(next_status.as_str())
    .bind(current_bid_id)
    .bind(current_price_amount)
    .bind(if current_price_amount.is_some() {
        Some(bid_currency.as_str())
    } else {
        None
    })
    .bind(winning_bid_id)
    .bind(winning_amount)
    .bind(winning_amount.map(|_| bid_currency.as_str()))
    .bind(auction_uuid)
    .execute(tx.as_mut())
    .await
    .map_err(db_error)?;

    let updated_row = sqlx::query("SELECT * FROM auctions WHERE id = $1")
        .bind(auction_uuid)
        .fetch_one(tx.as_mut())
        .await
        .map_err(db_error)?;
    let auction = auction_from_row(&updated_row);
    let bid = pb::AuctionBid {
        id: bid_id.to_string(),
        auction_id: auction_id.clone(),
        customer_id,
        amount: Some(money_from_parts(bid_amount, bid_currency.clone())),
        created_at: chrono_to_timestamp(Some(now)),
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            AuctionAuditAction::Bid.into(),
            Some("auction_bid"),
            Some(bid.id.clone()),
            None,
            to_json_opt(Some(bid.clone())),
            actor,
        ),
    )
    .await?;

    let auto_updated = apply_auto_bids_tx(&mut tx, &store_uuid.as_uuid(), auction_uuid).await?;
    let auction = auto_updated.unwrap_or(auction);

    tx.commit().await.map_err(db_error)?;

    Ok((auction, bid))
}

pub async fn set_auto_bid(
    state: &AppState,
    store_id: String,
    auction_id: String,
    customer_id: String,
    max_amount: Option<pb::Money>,
    enabled: bool,
    actor: Option<pb::ActorContext>,
) -> Result<(pb::Auction, pb::AuctionAutoBid), (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let auction_uuid = parse_uuid(&auction_id, "auction_id")?;
    let customer_uuid = parse_uuid(&customer_id, "customer_id")?;

    let mut tx = state.db.begin().await.map_err(db_error)?;
    let auction_row =
        sqlx::query("SELECT * FROM auctions WHERE id = $1 AND store_id = $2 FOR UPDATE")
            .bind(auction_uuid)
            .bind(store_uuid.as_uuid())
            .fetch_one(tx.as_mut())
            .await
            .map_err(db_error)?;
    let auction = auction_from_row(&auction_row);
    let auction_type: String = auction_row.get("auction_type");
    if auction_type != AUCTION_TYPE_OPEN {
        return Err(invalid_arg("auto bid is only available for open auctions"));
    }

    let start_price_amount: i64 = auction_row.get("start_price_amount");
    let start_price_currency: String = auction_row.get("start_price_currency");

    let (max_amount_value, max_currency) = if enabled {
        let (amount, currency) = money_to_parts(max_amount)?;
        if currency != start_price_currency {
            return Err(invalid_arg("currency mismatch"));
        }
        if amount < start_price_amount {
            return Err(invalid_arg("max_amount must be >= start_price"));
        }
        (amount, currency)
    } else {
        (0, start_price_currency.clone())
    };

    let eligible = sqlx::query(
        r#"
        SELECT 1
        FROM customer_profiles cp
        JOIN customers c ON c.id = cp.customer_id
        WHERE cp.store_id = $1
          AND cp.customer_id = $2
          AND cp.status = 'active'
          AND c.status = 'active'
        LIMIT 1
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(customer_uuid)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(db_error)?;
    if eligible.is_none() {
        return Err(permission_denied("customer is not eligible to bid"));
    }

    let status = if enabled {
        AUTO_BID_STATUS_ACTIVE
    } else {
        AUTO_BID_STATUS_DISABLED
    };

    let row = sqlx::query(
        r#"
        INSERT INTO auction_auto_bids (
            id, auction_id, store_id, customer_id, max_amount, currency, status
        ) VALUES ($1,$2,$3,$4,$5,$6,$7)
        ON CONFLICT (auction_id, customer_id)
        DO UPDATE SET max_amount = EXCLUDED.max_amount,
                      currency = EXCLUDED.currency,
                      status = EXCLUDED.status,
                      updated_at = now()
        RETURNING id, auction_id, customer_id, max_amount, currency, status, created_at, updated_at
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .bind(customer_uuid)
    .bind(max_amount_value)
    .bind(max_currency.as_str())
    .bind(status)
    .fetch_one(tx.as_mut())
    .await
    .map_err(db_error)?;

    let auto_bid = pb::AuctionAutoBid {
        id: row.get::<uuid::Uuid, _>("id").to_string(),
        auction_id: row.get::<uuid::Uuid, _>("auction_id").to_string(),
        customer_id: row.get::<uuid::Uuid, _>("customer_id").to_string(),
        max_amount: Some(money_from_parts(
            row.get::<i64, _>("max_amount"),
            row.get::<String, _>("currency"),
        )),
        status: row.get("status"),
        created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            AuctionAuditAction::Bid.into(),
            Some("auction_auto_bid"),
            Some(auto_bid.id.clone()),
            None,
            to_json_opt(Some(auto_bid.clone())),
            actor,
        ),
    )
    .await?;

    let auto_updated = apply_auto_bids_tx(&mut tx, &store_uuid.as_uuid(), auction_uuid).await?;
    let auction = auto_updated.unwrap_or(auction);

    tx.commit().await.map_err(db_error)?;

    Ok((auction, auto_bid))
}

pub async fn list_auto_bids(
    state: &AppState,
    store_id: String,
    auction_id: String,
) -> Result<Vec<pb::AuctionAutoBid>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let auction_uuid = parse_uuid(&auction_id, "auction_id")?;
    let rows = sqlx::query(
        r#"
        SELECT id, auction_id, customer_id, max_amount, currency, status, created_at, updated_at
        FROM auction_auto_bids
        WHERE auction_id = $1 AND store_id = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::AuctionAutoBid {
            id: row.get::<uuid::Uuid, _>("id").to_string(),
            auction_id: row.get::<uuid::Uuid, _>("auction_id").to_string(),
            customer_id: row.get::<uuid::Uuid, _>("customer_id").to_string(),
            max_amount: Some(money_from_parts(
                row.get::<i64, _>("max_amount"),
                row.get::<String, _>("currency"),
            )),
            status: row.get("status"),
            created_at: chrono_to_timestamp(Some(
                row.get::<chrono::DateTime<Utc>, _>("created_at"),
            )),
            updated_at: chrono_to_timestamp(Some(
                row.get::<chrono::DateTime<Utc>, _>("updated_at"),
            )),
        })
        .collect())
}

pub async fn run_scheduled_auctions(
    state: &AppState,
    batch_size: i64,
) -> Result<usize, (StatusCode, Json<ConnectError>)> {
    let mut tx = state.db.begin().await.map_err(db_error)?;
    let rows = sqlx::query(
        r#"
        WITH cte AS (
            SELECT id
            FROM auctions
            WHERE status = 'scheduled' AND start_at <= now()
            ORDER BY start_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE auctions AS a
        SET status = 'running',
            current_price_amount = COALESCE(a.current_price_amount, a.start_price_amount),
            current_price_currency = COALESCE(a.current_price_currency, a.start_price_currency),
            updated_at = now()
        FROM cte
        WHERE a.id = cte.id
        RETURNING a.id as id, a.store_id as store_id
        "#,
    )
    .bind(batch_size)
    .fetch_all(tx.as_mut())
    .await
    .map_err(db_error)?;
    tx.commit().await.map_err(db_error)?;

    for row in rows.iter() {
        let auction_id: uuid::Uuid = row.get("id");
        let store_id: uuid::Uuid = row.get("store_id");
        let mut tx = state.db.begin().await.map_err(db_error)?;
        let _ = apply_auto_bids_tx(&mut tx, &store_id, auction_id).await?;
        tx.commit().await.map_err(db_error)?;
    }

    Ok(rows.len())
}

async fn apply_auto_bids_tx(
    tx: &mut Transaction<'_, Postgres>,
    store_uuid: &uuid::Uuid,
    auction_uuid: uuid::Uuid,
) -> Result<Option<pb::Auction>, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT *
        FROM auctions
        WHERE id = $1 AND store_id = $2
        FOR UPDATE
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid)
    .fetch_one(tx.as_mut())
    .await
    .map_err(db_error)?;

    let auction_type: String = row.get("auction_type");
    let status: String = row.get("status");
    if auction_type != AUCTION_TYPE_OPEN || status != STATUS_RUNNING {
        return Ok(None);
    }

    let start_price_amount: i64 = row.get("start_price_amount");
    let start_price_currency: String = row.get("start_price_currency");
    let increment_amount: i64 = row.get("bid_increment_amount");
    let current_price_amount: Option<i64> = row.get("current_price_amount");
    let current_bid_id: Option<uuid::Uuid> = row.get("current_bid_id");
    let buyout_amount: Option<i64> = row.get("buyout_price_amount");
    let buyout_currency: Option<String> = row.get("buyout_price_currency");

    let current_bid_customer_id = if let Some(bid_id) = current_bid_id {
        sqlx::query("SELECT customer_id FROM auction_bids WHERE id = $1")
            .bind(bid_id)
            .fetch_optional(tx.as_mut())
            .await
            .map_err(db_error)?
            .and_then(|r| r.get::<Option<uuid::Uuid>, _>("customer_id"))
    } else {
        None
    };

    let candidates = sqlx::query(
        r#"
        SELECT id, customer_id, max_amount, currency, created_at
        FROM auction_auto_bids
        WHERE auction_id = $1
          AND store_id = $2
          AND status = $3
          AND currency = $4
        ORDER BY max_amount DESC, created_at ASC
        LIMIT 2
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid)
    .bind(AUTO_BID_STATUS_ACTIVE)
    .bind(start_price_currency.as_str())
    .fetch_all(tx.as_mut())
    .await
    .map_err(db_error)?;

    if candidates.is_empty() {
        return Ok(None);
    }

    let to_candidate = |row: &sqlx::postgres::PgRow| AutoBidCandidate {
        id: row.get("id"),
        customer_id: row.get("customer_id"),
        max_amount: row.get("max_amount"),
        created_at: row.get("created_at"),
    };
    let top = to_candidate(&candidates[0]);
    if top.max_amount < start_price_amount {
        return Ok(None);
    }
    let second = candidates.get(1).map(to_candidate);

    let has_current_bid = current_bid_id.is_some();
    let current_amount = current_price_amount.unwrap_or(0);
    let min_next = if has_current_bid {
        current_amount + increment_amount
    } else {
        start_price_amount
    };
    let mut target_amount = if let Some(second) = second {
        std::cmp::min(top.max_amount, second.max_amount + increment_amount)
    } else {
        std::cmp::min(top.max_amount, min_next)
    };
    if target_amount < start_price_amount {
        target_amount = start_price_amount;
    }
    let compare_amount = if has_current_bid { current_amount } else { 0 };
    if target_amount <= compare_amount {
        return Ok(None);
    }
    if current_bid_customer_id == Some(top.customer_id) && current_amount >= target_amount {
        return Ok(None);
    }

    let bid_id = uuid::Uuid::new_v4();
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO auction_bids
            (id, auction_id, store_id, customer_id, amount, currency)
        VALUES ($1,$2,$3,$4,$5,$6)
        "#,
    )
    .bind(bid_id)
    .bind(auction_uuid)
    .bind(store_uuid)
    .bind(top.customer_id)
    .bind(target_amount)
    .bind(start_price_currency.as_str())
    .execute(tx.as_mut())
    .await
    .map_err(db_error)?;

    let mut next_status = status.clone();
    if let (Some(buyout), Some(cur)) = (buyout_amount, buyout_currency.clone())
        && cur == start_price_currency && target_amount >= buyout {
            next_status = STATUS_AWAITING_APPROVAL.to_string();
        }

    sqlx::query(
        r#"
        UPDATE auctions
        SET status = $1,
            current_bid_id = $2,
            current_price_amount = $3,
            current_price_currency = $4,
            winning_bid_id = $5,
            winning_price_amount = $6,
            winning_price_currency = $7,
            updated_at = now()
        WHERE id = $8
        "#,
    )
    .bind(next_status.as_str())
    .bind(bid_id)
    .bind(target_amount)
    .bind(start_price_currency.as_str())
    .bind(bid_id)
    .bind(target_amount)
    .bind(start_price_currency.as_str())
    .bind(auction_uuid)
    .execute(tx.as_mut())
    .await
    .map_err(db_error)?;

    let updated_row = sqlx::query("SELECT * FROM auctions WHERE id = $1")
        .bind(auction_uuid)
        .fetch_one(tx.as_mut())
        .await
        .map_err(db_error)?;
    let auction = auction_from_row(&updated_row);
    let bid = pb::AuctionBid {
        id: bid_id.to_string(),
        auction_id: auction.id.clone(),
        customer_id: top.customer_id.to_string(),
        amount: Some(money_from_parts(
            target_amount,
            start_price_currency.clone(),
        )),
        created_at: chrono_to_timestamp(Some(now)),
    };

    audit::record_tx(
        tx,
        audit_input(
            Some(auction.store_id.clone()),
            AuctionAuditAction::Bid.into(),
            Some("auction_auto_bid"),
            Some(bid.id.clone()),
            None,
            to_json_opt(Some(bid)),
            None,
        ),
    )
    .await?;

    Ok(Some(auction))
}

pub async fn close_auction(
    state: &AppState,
    store_id: String,
    auction_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<pb::Auction, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let auction_uuid = parse_uuid(&auction_id, "auction_id")?;

    let mut tx = state.db.begin().await.map_err(db_error)?;
    let row = sqlx::query(
        r#"
        SELECT *
        FROM auctions
        WHERE id = $1 AND store_id = $2
        FOR UPDATE
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_one(tx.as_mut())
    .await
    .map_err(db_error)?;

    let reserve_amount: Option<i64> = row.get("reserve_price_amount");
    let reserve_currency: Option<String> = row.get("reserve_price_currency");

    let best_bid = sqlx::query(
        r#"
        SELECT id, amount, currency
        FROM auction_bids
        WHERE auction_id = $1 AND store_id = $2
        ORDER BY amount DESC, created_at ASC
        LIMIT 1
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(tx.as_mut())
    .await
    .map_err(db_error)?;

    let (winning_bid_id, winning_amount, winning_currency) = if let Some(bid) = best_bid {
        (
            Some(bid.get::<uuid::Uuid, _>("id")),
            Some(bid.get::<i64, _>("amount")),
            Some(bid.get::<String, _>("currency")),
        )
    } else {
        (None, None, None)
    };

    let mut next_status = STATUS_ENDED.to_string();
    if let (Some(win_amount), Some(win_currency)) = (winning_amount, winning_currency.clone())
        && (reserve_amount.is_none()
            || (reserve_currency.as_deref() == Some(win_currency.as_str())
                && reserve_amount.unwrap_or(0) <= win_amount))
        {
            next_status = STATUS_AWAITING_APPROVAL.to_string();
        }

    sqlx::query(
        r#"
        UPDATE auctions
        SET status = $1,
            winning_bid_id = $2,
            winning_price_amount = $3,
            winning_price_currency = $4,
            end_at = now(),
            updated_at = now()
        WHERE id = $5
        "#,
    )
    .bind(next_status.as_str())
    .bind(winning_bid_id)
    .bind(winning_amount)
    .bind(winning_currency.as_deref())
    .bind(auction_uuid)
    .execute(tx.as_mut())
    .await
    .map_err(db_error)?;

    let updated_row = sqlx::query("SELECT * FROM auctions WHERE id = $1")
        .bind(auction_uuid)
        .fetch_one(tx.as_mut())
        .await
        .map_err(db_error)?;
    let auction = auction_from_row(&updated_row);

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            AuctionAuditAction::End.into(),
            Some("auction"),
            Some(auction_id),
            None,
            to_json_opt(Some(auction.clone())),
            actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db_error)?;

    Ok(auction)
}

pub async fn approve_auction(
    state: &AppState,
    store_id: String,
    auction_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<pb::Auction, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let auction_uuid = parse_uuid(&auction_id, "auction_id")?;

    let approved_by = actor
        .as_ref()
        .and_then(|a| {
            if a.actor_id.is_empty() {
                None
            } else {
                Some(a.actor_id.as_str())
            }
        })
        .map(|id| parse_uuid(id, "approved_by"))
        .transpose()?;

    let mut tx = state.db.begin().await.map_err(db_error)?;
    let row = sqlx::query(
        r#"
        SELECT *
        FROM auctions
        WHERE id = $1 AND store_id = $2
        FOR UPDATE
        "#,
    )
    .bind(auction_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_one(tx.as_mut())
    .await
    .map_err(db_error)?;

    let status: String = row.get("status");
    if status != STATUS_AWAITING_APPROVAL {
        return Err(invalid_arg("auction is not awaiting approval"));
    }

    sqlx::query(
        r#"
        UPDATE auctions
        SET status = $1,
            approved_by = $2,
            approved_at = now(),
            updated_at = now()
        WHERE id = $3
        "#,
    )
    .bind(STATUS_APPROVED)
    .bind(approved_by)
    .bind(auction_uuid)
    .execute(tx.as_mut())
    .await
    .map_err(db_error)?;

    let updated_row = sqlx::query("SELECT * FROM auctions WHERE id = $1")
        .bind(auction_uuid)
        .fetch_one(tx.as_mut())
        .await
        .map_err(db_error)?;
    let auction = auction_from_row(&updated_row);

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            AuctionAuditAction::Approve.into(),
            Some("auction"),
            Some(auction_id),
            None,
            to_json_opt(Some(auction.clone())),
            actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db_error)?;

    Ok(auction)
}

fn auction_from_row(row: &sqlx::postgres::PgRow) -> pb::Auction {
    let reserve_amount: Option<i64> = row.get("reserve_price_amount");
    let reserve_currency: Option<String> = row.get("reserve_price_currency");
    let buyout_amount: Option<i64> = row.get("buyout_price_amount");
    let buyout_currency: Option<String> = row.get("buyout_price_currency");
    let current_amount: Option<i64> = row.get("current_price_amount");
    let current_currency: Option<String> = row.get("current_price_currency");
    let winning_amount: Option<i64> = row.get("winning_price_amount");
    let winning_currency: Option<String> = row.get("winning_price_currency");
    pb::Auction {
        id: row.get::<uuid::Uuid, _>("id").to_string(),
        store_id: row.get::<uuid::Uuid, _>("store_id").to_string(),
        product_id: row
            .get::<Option<uuid::Uuid>, _>("product_id")
            .map(|v| v.to_string())
            .unwrap_or_default(),
        sku_id: row.get::<uuid::Uuid, _>("sku_id").to_string(),
        auction_type: row.get("auction_type"),
        status: row.get("status"),
        start_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("start_at"))),
        end_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("end_at"))),
        bid_increment: Some(money_from_parts(
            row.get::<i64, _>("bid_increment_amount"),
            row.get::<String, _>("bid_increment_currency"),
        )),
        start_price: Some(money_from_parts(
            row.get::<i64, _>("start_price_amount"),
            row.get::<String, _>("start_price_currency"),
        )),
        reserve_price: reserve_amount
            .zip(reserve_currency)
            .map(|(amt, cur)| money_from_parts(amt, cur)),
        buyout_price: buyout_amount
            .zip(buyout_currency)
            .map(|(amt, cur)| money_from_parts(amt, cur)),
        current_price: current_amount
            .zip(current_currency)
            .map(|(amt, cur)| money_from_parts(amt, cur)),
        winning_price: winning_amount
            .zip(winning_currency)
            .map(|(amt, cur)| money_from_parts(amt, cur)),
        winning_bid_id: row
            .get::<Option<uuid::Uuid>, _>("winning_bid_id")
            .map(|v| v.to_string())
            .unwrap_or_default(),
        approved_by: row
            .get::<Option<uuid::Uuid>, _>("approved_by")
            .map(|v| v.to_string())
            .unwrap_or_default(),
        approved_at: chrono_to_timestamp(
            row.get::<Option<chrono::DateTime<Utc>>, _>("approved_at"),
        ),
        created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
        title: row.get::<String, _>("title"),
        description: row.get::<String, _>("description"),
    }
}

fn invalid_arg(message: &str) -> (StatusCode, Json<ConnectError>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ConnectError {
            code: crate::rpc::json::ErrorCode::InvalidArgument,
            message: message.to_string(),
        }),
    )
}

fn permission_denied(message: &str) -> (StatusCode, Json<ConnectError>) {
    (
        StatusCode::FORBIDDEN,
        Json(ConnectError {
            code: crate::rpc::json::ErrorCode::PermissionDenied,
            message: message.to_string(),
        }),
    )
}

fn db_error(err: impl std::fmt::Display) -> (StatusCode, Json<ConnectError>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ConnectError {
            code: crate::rpc::json::ErrorCode::Internal,
            message: format!("db error: {}", err),
        }),
    )
}

pub async fn resolve_context(
    state: &AppState,
    store: Option<pb::StoreContext>,
) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, None).await?;
    Ok(store_id)
}
