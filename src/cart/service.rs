use chrono::{Duration, Utc};
use sqlx::Row;

use crate::{
    AppState,
    cart::error::{CartError, CartResult},
    pb::pb,
    shared::ids::{CartId, CartItemId, CustomerId, LocationId, SkuId, StoreId, parse_uuid},
    shared::status::{CartItemStatus, CartStatus, PaymentMethod},
    shared::time::chrono_to_timestamp,
};

fn cart_ttl_days() -> i64 {
    std::env::var("CART_TTL_DAYS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(30)
}

async fn resolve_store_id(state: &AppState, store: Option<pb::StoreContext>) -> CartResult<String> {
    let (store_id, _tenant_id) =
        crate::identity::context::resolve_store_context_without_token_guard(state, store, None)
            .await?;
    Ok(store_id)
}

pub async fn create_cart(state: &AppState, req: pb::CreateCartRequest) -> CartResult<pb::Cart> {
    let cart_id = uuid::Uuid::new_v4();
    let store_id = resolve_store_id(state, req.store.clone()).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let expires_at = Utc::now() + Duration::days(cart_ttl_days());
    sqlx::query(
        r#"
        INSERT INTO carts (id, store_id, customer_id, status, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(cart_id)
    .bind(store_uuid.as_uuid())
    .bind(if req.customer_id.is_empty() {
        None
    } else {
        Some(CustomerId::parse(&req.customer_id)?.as_uuid())
    })
    .bind(CartStatus::Active.as_str())
    .bind(expires_at)
    .execute(&state.db)
    .await
    .map_err(CartError::from)?;

    Ok(pb::Cart {
        id: cart_id.to_string(),
        store_id,
        customer_id: req.customer_id,
        items: Vec::new(),
        total: None,
        status: CartStatus::Active.as_str().to_string(),
        expires_at: chrono_to_timestamp(Some(expires_at)),
    })
}

pub async fn add_cart_item(state: &AppState, req: pb::AddCartItemRequest) -> CartResult<pb::Cart> {
    if req.quantity <= 0 {
        return Err(CartError::invalid_argument(
            "quantity must be greater than 0",
        ));
    }

    let store_id = resolve_store_id(state, req.store.clone()).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let cart_uuid = CartId::parse(&req.cart_id)?;
    let sku_uuid = SkuId::parse(&req.sku_id)?;
    let location_uuid = if req.location_id.is_empty() {
        None
    } else {
        Some(LocationId::parse(&req.location_id)?)
    };

    // Validate cart ownership.
    let cart_exists =
        sqlx::query("SELECT id, expires_at FROM carts WHERE id = $1 AND store_id = $2 LIMIT 1")
            .bind(cart_uuid.as_uuid())
            .bind(store_uuid.as_uuid())
            .fetch_optional(&state.db)
            .await
            .map_err(CartError::from)?;
    let Some(cart_row) = cart_exists else {
        return Err(CartError::not_found("cart not found"));
    };
    let expires_at: chrono::DateTime<Utc> = cart_row.get("expires_at");

    // Fetch SKU price + fulfillment type via product.
    let row = sqlx::query(
        r#"
        SELECT v.price_amount, v.price_currency, v.fulfillment_type
        FROM product_skus v
        JOIN products p ON p.id = v.product_id
        WHERE v.id = $1 AND p.store_id = $2
        LIMIT 1
        "#,
    )
    .bind(sku_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(CartError::from)?;
    let Some(row) = row else {
        return Err(CartError::not_found("variant not found"));
    };

    let price_amount: i64 = row.get("price_amount");
    let price_currency: String = row.get("price_currency");
    let fulfillment_type: String = row.get("fulfillment_type");
    let is_physical = fulfillment_type == "physical";
    if is_physical && location_uuid.is_none() {
        return Err(CartError::invalid_argument(
            "location_id is required for physical SKU",
        ));
    }

    let cart_item_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO cart_items (
            id, cart_id, sku_id, location_id, unit_price_amount, unit_price_currency,
            quantity, fulfillment_type, status
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        "#,
    )
    .bind(cart_item_id)
    .bind(cart_uuid.as_uuid())
    .bind(sku_uuid.as_uuid())
    .bind(location_uuid.map(|value| value.as_uuid()))
    .bind(price_amount)
    .bind(&price_currency)
    .bind(req.quantity)
    .bind(&fulfillment_type)
    .bind(CartItemStatus::Active.as_str())
    .execute(&state.db)
    .await
    .map_err(CartError::from)?;

    // Enqueue reservation request (async worker) for physical items only.
    if is_physical {
        let request_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO inventory_reservation_requests (
                id, store_id, cart_id, cart_item_id, sku_id, location_id, quantity, status, is_hot
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,'queued',false)
            "#,
        )
        .bind(request_id)
        .bind(store_uuid.as_uuid())
        .bind(cart_uuid.as_uuid())
        .bind(cart_item_id)
        .bind(sku_uuid.as_uuid())
        .bind(location_uuid.map(|value| value.as_uuid()))
        .bind(req.quantity)
        .execute(&state.db)
        .await
        .map_err(CartError::from)?;
    }

    Ok(pb::Cart {
        id: cart_uuid.to_string(),
        store_id,
        customer_id: String::new(),
        items: vec![pb::CartItem {
            id: cart_item_id.to_string(),
            sku_id: sku_uuid.to_string(),
            location_id: location_uuid.map(|id| id.to_string()).unwrap_or_default(),
            unit_price: Some(pb::Money {
                amount: price_amount,
                currency: price_currency.clone(),
            }),
            quantity: req.quantity,
            fulfillment_type: fulfillment_type.clone(),
            status: CartItemStatus::Active.as_str().to_string(),
        }],
        total: Some(pb::Money {
            amount: price_amount.saturating_mul(req.quantity as i64),
            currency: price_currency,
        }),
        status: CartStatus::Active.as_str().to_string(),
        expires_at: chrono_to_timestamp(Some(expires_at)),
    })
}

pub async fn remove_cart_item(
    state: &AppState,
    req: pb::RemoveCartItemRequest,
) -> CartResult<pb::Cart> {
    let store_id = resolve_store_id(state, req.store.clone()).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let cart_item_uuid = CartItemId::parse(&req.cart_item_id)?;

    let mut tx = state.db.begin().await.map_err(CartError::from)?;

    let row = sqlx::query(
        r#"
        SELECT c.id::text as cart_id, c.expires_at, ci.sku_id::text as sku_id, ci.location_id::text as location_id,
               ci.quantity, ci.fulfillment_type
        FROM cart_items ci
        JOIN carts c ON c.id = ci.cart_id
        WHERE ci.id = $1 AND c.store_id = $2
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(cart_item_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(CartError::from)?;

    let Some(row) = row else {
        return Err(CartError::not_found("cart item not found"));
    };

    let cart_id: String = row.get("cart_id");
    let sku_id: String = row.get("sku_id");
    let location_id: Option<String> = row.try_get("location_id").ok();
    let quantity: i32 = row.get("quantity");
    let fulfillment_type: String = row.get("fulfillment_type");
    let expires_at: chrono::DateTime<Utc> = row.get("expires_at");

    if fulfillment_type == "physical" {
        sqlx::query(
            r#"
            UPDATE inventory_reservations
            SET status = 'released', updated_at = now()
            WHERE cart_item_id = $1 AND status = 'active'
            "#,
        )
        .bind(cart_item_uuid.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(CartError::from)?;

        let sku_uuid = SkuId::parse(&sku_id)?;
        let location_uuid = location_id
            .as_deref()
            .map(|value| LocationId::parse(value))
            .transpose()?;
        if let Some(location_uuid) = location_uuid {
            sqlx::query(
                r#"
                UPDATE inventory_stocks
                SET reserved = GREATEST(reserved - $1, 0),
                    updated_at = now()
                WHERE variant_id = $2 AND location_id = $3
                "#,
            )
            .bind(quantity)
            .bind(sku_uuid.as_uuid())
            .bind(location_uuid.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(CartError::from)?;
        }
    }

    sqlx::query("UPDATE cart_items SET status = $1, updated_at = now() WHERE id = $2")
        .bind(CartItemStatus::Removed.as_str())
        .bind(cart_item_uuid.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(CartError::from)?;

    tx.commit().await.map_err(CartError::from)?;

    Ok(pb::Cart {
        id: cart_id,
        store_id,
        customer_id: String::new(),
        items: Vec::new(),
        total: None,
        status: CartStatus::Active.as_str().to_string(),
        expires_at: chrono_to_timestamp(Some(expires_at)),
    })
}

pub async fn update_cart_item(
    state: &AppState,
    req: pb::UpdateCartItemRequest,
) -> CartResult<pb::Cart> {
    if req.quantity <= 0 {
        return Err(CartError::invalid_argument(
            "quantity must be greater than 0",
        ));
    }

    let store_id = resolve_store_id(state, req.store.clone()).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let cart_item_uuid = CartItemId::parse(&req.cart_item_id)?;

    let mut tx = state.db.begin().await.map_err(CartError::from)?;
    let row = sqlx::query(
        r#"
        SELECT c.id::text as cart_id, c.expires_at, ci.sku_id::text as sku_id, ci.location_id::text as location_id,
               ci.quantity, ci.fulfillment_type
        FROM cart_items ci
        JOIN carts c ON c.id = ci.cart_id
        WHERE ci.id = $1 AND c.store_id = $2
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(cart_item_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(CartError::from)?;

    let Some(row) = row else {
        return Err(CartError::not_found("cart item not found"));
    };

    let cart_id: String = row.get("cart_id");
    let sku_id: String = row.get("sku_id");
    let location_id: Option<String> = row.try_get("location_id").ok();
    let current_qty: i32 = row.get("quantity");
    let fulfillment_type: String = row.get("fulfillment_type");
    let expires_at: chrono::DateTime<Utc> = row.get("expires_at");

    if req.quantity == current_qty {
        tx.commit().await.map_err(CartError::from)?;
        return Ok(pb::Cart {
            id: cart_id,
            store_id,
            customer_id: String::new(),
            items: Vec::new(),
            total: None,
            status: CartStatus::Active.as_str().to_string(),
            expires_at: chrono_to_timestamp(Some(expires_at)),
        });
    }

    sqlx::query("UPDATE cart_items SET quantity = $1, updated_at = now() WHERE id = $2")
        .bind(req.quantity)
        .bind(cart_item_uuid.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(CartError::from)?;

    let delta = req.quantity - current_qty;
    if fulfillment_type == "physical" {
        let sku_uuid = SkuId::parse(&sku_id)?;
        let location_uuid = location_id
            .as_deref()
            .map(|value| LocationId::parse(value))
            .transpose()?;
        if delta > 0 {
            if let Some(location_uuid) = location_uuid {
                let request_id = uuid::Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO inventory_reservation_requests (
                        id, store_id, cart_id, cart_item_id, sku_id, location_id, quantity, status, is_hot
                    )
                    VALUES ($1,$2,$3,$4,$5,$6,$7,'queued',false)
                    "#,
                )
                .bind(request_id)
                .bind(store_uuid.as_uuid())
                .bind(CartId::parse(&cart_id)?.as_uuid())
                .bind(cart_item_uuid.as_uuid())
                .bind(sku_uuid.as_uuid())
                .bind(location_uuid.as_uuid())
                .bind(delta)
                .execute(&mut *tx)
                .await
                .map_err(CartError::from)?;
            }
        } else {
            let release_qty = -delta;
            let reservation = sqlx::query(
                r#"
                SELECT id::text as id, quantity
                FROM inventory_reservations
                WHERE cart_item_id = $1 AND status = 'active'
                LIMIT 1
                FOR UPDATE
                "#,
            )
            .bind(cart_item_uuid.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(CartError::from)?;

            if let Some(reservation) = reservation {
                let reservation_id: String = reservation.get("id");
                let reserved_qty: i32 = reservation.get("quantity");
                let new_reserved = (reserved_qty - release_qty).max(0);

                if new_reserved == 0 {
                    sqlx::query(
                        r#"
                        UPDATE inventory_reservations
                        SET status = 'released', updated_at = now()
                        WHERE id = $1
                        "#,
                    )
                    .bind(parse_uuid(&reservation_id, "reservation_id")?)
                    .execute(&mut *tx)
                    .await
                    .map_err(CartError::from)?;
                } else {
                    sqlx::query(
                        r#"
                        UPDATE inventory_reservations
                        SET quantity = $1, updated_at = now()
                        WHERE id = $2
                        "#,
                    )
                    .bind(new_reserved)
                    .bind(parse_uuid(&reservation_id, "reservation_id")?)
                    .execute(&mut *tx)
                    .await
                    .map_err(CartError::from)?;
                }
            }

            if let Some(location_uuid) = location_uuid {
                sqlx::query(
                    r#"
                    UPDATE inventory_stocks
                    SET reserved = GREATEST(reserved - $1, 0),
                        updated_at = now()
                    WHERE variant_id = $2 AND location_id = $3
                    "#,
                )
                .bind(release_qty)
                .bind(sku_uuid.as_uuid())
                .bind(location_uuid.as_uuid())
                .execute(&mut *tx)
                .await
                .map_err(CartError::from)?;
            }
        }
    }

    tx.commit().await.map_err(CartError::from)?;

    Ok(pb::Cart {
        id: cart_id,
        store_id,
        customer_id: String::new(),
        items: Vec::new(),
        total: None,
        status: CartStatus::Active.as_str().to_string(),
        expires_at: chrono_to_timestamp(Some(expires_at)),
    })
}

pub async fn get_cart(state: &AppState, req: pb::GetCartRequest) -> CartResult<pb::Cart> {
    let store_id = resolve_store_id(state, req.store.clone()).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let cart_uuid = CartId::parse(&req.cart_id)?;

    let cart_row = sqlx::query(
        r#"
        SELECT customer_id::text as customer_id, status, expires_at
        FROM carts
        WHERE id = $1 AND store_id = $2
        LIMIT 1
        "#,
    )
    .bind(cart_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(CartError::from)?;
    let Some(cart_row) = cart_row else {
        return Err(CartError::not_found("cart not found"));
    };

    let items = sqlx::query(
        r#"
        SELECT id::text as id, sku_id::text as sku_id, location_id::text as location_id,
               unit_price_amount, unit_price_currency, quantity, fulfillment_type, status
        FROM cart_items
        WHERE cart_id = $1 AND status = 'active'
        ORDER BY created_at ASC
        "#,
    )
    .bind(cart_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(CartError::from)?;

    let mut total_amount: i64 = 0;
    let mut currency: Option<String> = None;
    let mut has_mixed_currency = false;

    let cart_items = items
        .into_iter()
        .map(|row| {
            let price_amount: i64 = row.get("unit_price_amount");
            let price_currency: String = row.get("unit_price_currency");
            let quantity: i32 = row.get("quantity");

            if let Some(curr) = &currency {
                if curr != &price_currency {
                    has_mixed_currency = true;
                }
            } else {
                currency = Some(price_currency.clone());
            }
            total_amount = total_amount.saturating_add(price_amount * (quantity as i64));

            pb::CartItem {
                id: row.get("id"),
                sku_id: row.get("sku_id"),
                location_id: row
                    .get::<Option<String>, _>("location_id")
                    .unwrap_or_default(),
                unit_price: Some(pb::Money {
                    amount: price_amount,
                    currency: price_currency,
                }),
                quantity,
                fulfillment_type: row.get("fulfillment_type"),
                status: row.get("status"),
            }
        })
        .collect::<Vec<_>>();

    let total = if has_mixed_currency || currency.is_none() {
        None
    } else {
        Some(pb::Money {
            amount: total_amount,
            currency: currency.unwrap(),
        })
    };

    Ok(pb::Cart {
        id: cart_uuid.to_string(),
        store_id,
        customer_id: cart_row
            .get::<Option<String>, _>("customer_id")
            .unwrap_or_default(),
        items: cart_items,
        total,
        status: cart_row.get("status"),
        expires_at: chrono_to_timestamp(Some(cart_row.get("expires_at"))),
    })
}

pub async fn checkout(
    state: &AppState,
    tenant_id: String,
    req: pb::CheckoutRequest,
) -> CartResult<pb::Order> {
    let tenant_uuid = parse_uuid(&tenant_id, "tenant_id")?;
    let cart_uuid = CartId::parse(&req.cart_id)?;
    let payment_method = PaymentMethod::from_pb(req.payment_method)?;

    let mut tx = state.db.begin().await.map_err(CartError::from)?;
    let store_row = sqlx::query(
        "SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(tenant_uuid)
    .fetch_optional(&mut *tx)
    .await
    .map_err(CartError::from)?;
    let Some(store_row) = store_row else {
        return Err(CartError::invalid_argument("tenant_id not found"));
    };
    let store_id: String = store_row.get("id");
    let store_uuid = StoreId::parse(&store_id)?;

    let cart_row = sqlx::query(
        "SELECT customer_id::text as customer_id FROM carts WHERE id = $1 AND store_id = $2 LIMIT 1 FOR UPDATE",
    )
    .bind(cart_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(CartError::from)?;
    let Some(cart_row) = cart_row else {
        return Err(CartError::not_found("cart not found"));
    };

    let items = sqlx::query(
        r#"
        SELECT id::text as cart_item_id, sku_id::text as sku_id,
               location_id::text as location_id, unit_price_amount, unit_price_currency,
               quantity, fulfillment_type
        FROM cart_items
        WHERE cart_id = $1
        FOR UPDATE
        "#,
    )
    .bind(cart_uuid.as_uuid())
    .fetch_all(&mut *tx)
    .await
    .map_err(CartError::from)?;

    if items.is_empty() {
        return Err(CartError::invalid_argument("cart has no items"));
    }

    let mut total_amount: i64 = 0;
    let mut currency: Option<String> = None;

    for item in &items {
        let cart_item_id: String = item.get("cart_item_id");
        let sku_id: String = item.get("sku_id");
        let location_id: Option<String> = item.try_get("location_id").ok();
        let quantity: i32 = item.get("quantity");
        let price_amount: i64 = item.get("unit_price_amount");
        let price_currency: String = item.get("unit_price_currency");
        let fulfillment_type: String = item.get("fulfillment_type");
        let is_physical = fulfillment_type == "physical";

        if let Some(curr) = &currency {
            if curr != &price_currency {
                return Err(CartError::invalid_argument(
                    "mixed currency cart is not supported",
                ));
            }
        } else {
            currency = Some(price_currency.clone());
        }

        if is_physical {
            let reservation = sqlx::query(
                r#"
                SELECT id::text as id, quantity
                FROM inventory_reservations
                WHERE cart_item_id = $1 AND status = 'active'
                LIMIT 1
                FOR UPDATE
                "#,
            )
            .bind(parse_uuid(&cart_item_id, "cart_item_id")?)
            .fetch_optional(&mut *tx)
            .await
            .map_err(CartError::from)?;

            let Some(reservation) = reservation else {
                return Err(CartError::failed_precondition(
                    "inventory reservation not ready",
                ));
            };
            let reserved_qty: i32 = reservation.get("quantity");
            if reserved_qty < quantity {
                return Err(CartError::failed_precondition(
                    "reserved quantity is insufficient",
                ));
            }

            let sku_uuid = parse_uuid(&sku_id, "sku_id")?;
            let location_uuid = location_id
                .as_deref()
                .map(|value| parse_uuid(value, "location_id"))
                .transpose()?;
            if let Some(location_uuid) = location_uuid {
                let updated = sqlx::query(
                    r#"
                    UPDATE inventory_stocks
                    SET stock = stock - $1,
                        reserved = reserved - $1,
                        updated_at = now()
                    WHERE variant_id = $2
                      AND location_id = $3
                      AND stock >= $1
                      AND reserved >= $1
                    "#,
                )
                .bind(quantity)
                .bind(sku_uuid)
                .bind(location_uuid)
                .execute(&mut *tx)
                .await
                .map_err(CartError::from)?;
                if updated.rows_affected() != 1 {
                    return Err(CartError::failed_precondition(
                        "inventory stock is insufficient",
                    ));
                }
            }

            let reservation_id: String = reservation.get("id");
            sqlx::query(
                r#"
                UPDATE inventory_reservations
                SET status = 'consumed', updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(parse_uuid(&reservation_id, "reservation_id")?)
            .execute(&mut *tx)
            .await
            .map_err(CartError::from)?;
        }

        total_amount = total_amount.saturating_add(price_amount * (quantity as i64));
    }

    let order_id = uuid::Uuid::new_v4();
    let status = match payment_method {
        PaymentMethod::BankTransfer => "pending_payment",
        PaymentMethod::Cod => "pending_shipment",
    };
    sqlx::query(
        r#"
        INSERT INTO orders (id, tenant_id, customer_id, status, total_amount, currency, payment_method)
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        "#,
    )
    .bind(order_id)
    .bind(tenant_uuid)
    .bind(cart_row.get::<Option<String>, _>("customer_id").and_then(|id| parse_uuid(&id, "customer_id").ok()))
    .bind(status)
    .bind(total_amount)
    .bind(currency.clone().unwrap_or_else(|| "JPY".to_string()))
    .bind(payment_method.as_str())
    .execute(&mut *tx)
    .await
    .map_err(CartError::from)?;

    for item in &items {
        let cart_item_id: String = item.get("cart_item_id");
        let sku_id: String = item.get("sku_id");
        let vendor_id: Option<String> = None;
        let quantity: i32 = item.get("quantity");
        let price_amount: i64 = item.get("unit_price_amount");
        let price_currency: String = item.get("unit_price_currency");

        sqlx::query(
            r#"
            INSERT INTO order_items (id, order_id, vendor_id, variant_id, price_amount, price_currency, quantity)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(order_id)
        .bind(vendor_id.as_deref().and_then(|id| parse_uuid(id, "vendor_id").ok()))
        .bind(parse_uuid(&sku_id, "sku_id")?)
        .bind(price_amount)
        .bind(price_currency)
        .bind(quantity)
        .execute(&mut *tx)
        .await
        .map_err(CartError::from)?;

        sqlx::query("DELETE FROM cart_items WHERE id = $1")
            .bind(parse_uuid(&cart_item_id, "cart_item_id")?)
            .execute(&mut *tx)
            .await
            .map_err(CartError::from)?;
    }

    sqlx::query("UPDATE carts SET status = $1, updated_at = now() WHERE id = $2")
        .bind(CartStatus::Ordered.as_str())
        .bind(cart_uuid.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(CartError::from)?;

    tx.commit().await.map_err(CartError::from)?;

    Ok(pb::Order {
        id: order_id.to_string(),
        customer_id: cart_row
            .get::<Option<String>, _>("customer_id")
            .unwrap_or_default(),
        status: match status {
            "pending_payment" => pb::OrderStatus::PendingPayment as i32,
            "pending_shipment" => pb::OrderStatus::PendingShipment as i32,
            _ => pb::OrderStatus::Unspecified as i32,
        },
        total: Some(pb::Money {
            amount: total_amount,
            currency: currency.unwrap_or_else(|| "JPY".to_string()),
        }),
        payment_method: req.payment_method,
        shipping_address: req.shipping_address,
        billing_address: req.billing_address,
        created_at: None,
    })
}
