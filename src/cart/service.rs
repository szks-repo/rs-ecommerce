use axum::{Json, http::StatusCode};
use sqlx::Row;

use crate::{
    AppState,
    pb::pb,
    infrastructure::db,
    rpc::json::ConnectError,
    shared::ids::parse_uuid,
    shared::status::payment_method_to_string,
};

pub async fn create_cart(
    state: &AppState,
    tenant_id: String,
    req: pb::CreateCartRequest,
) -> Result<pb::Cart, (StatusCode, Json<ConnectError>)> {
    let cart_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO carts (id, tenant_id, customer_id, status)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(cart_id)
    .bind(&tenant_id)
    .bind(parse_uuid(&req.customer_id, "customer_id").ok())
    .bind("active")
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::Cart {
        id: cart_id.to_string(),
        customer_id: String::new(),
        items: Vec::new(),
        total: None,
        status: "active".to_string(),
    })
}

pub async fn add_cart_item(
    state: &AppState,
    tenant_id: String,
    req: pb::AddCartItemRequest,
) -> Result<pb::Cart, (StatusCode, Json<ConnectError>)> {
    if req.quantity <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "quantity must be greater than 0".to_string(),
            }),
        ));
    }

    let tenant_uuid = parse_uuid(&tenant_id, "tenant_id")?;
    let cart_uuid = parse_uuid(&req.cart_id, "cart_id")?;
    let variant_uuid = parse_uuid(&req.variant_id, "variant_id")?;

    // Resolve store_id for the tenant (single-brand default store).
    let store_row = sqlx::query("SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1")
        .bind(tenant_uuid)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
    let store_id: String = store_row.get("id");
    let store_uuid = parse_uuid(&store_id, "store_id")?;

    // Validate cart ownership.
    let cart_exists = sqlx::query(
        "SELECT id FROM carts WHERE id = $1 AND tenant_id = $2 LIMIT 1",
    )
    .bind(cart_uuid)
    .bind(tenant_uuid)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    if cart_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: "not_found",
                message: "cart not found".to_string(),
            }),
        ));
    }

    // Fetch variant price + vendor_id via product.
    let row = sqlx::query(
        r#"
        SELECT p.vendor_id, v.price_amount, v.price_currency
        FROM variants v
        JOIN products p ON p.id = v.product_id
        WHERE v.id = $1
        LIMIT 1
        "#,
    )
    .bind(variant_uuid)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: "not_found",
                message: "variant not found".to_string(),
            }),
        ));
    };

    let vendor_id: Option<uuid::Uuid> = row.try_get("vendor_id").ok();
    let price_amount: i64 = row.get("price_amount");
    let price_currency: String = row.get("price_currency");

    let cart_item_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO cart_items (id, cart_id, vendor_id, variant_id, price_amount, price_currency, quantity)
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        "#,
    )
    .bind(cart_item_id)
    .bind(cart_uuid)
    .bind(vendor_id)
    .bind(variant_uuid)
    .bind(price_amount)
    .bind(&price_currency)
    .bind(req.quantity)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    // Enqueue reservation request (async worker).
    let request_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO inventory_reservation_requests (
            id, tenant_id, store_id, cart_id, cart_item_id, variant_id, quantity, status, is_hot
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,'queued',false)
        "#,
    )
    .bind(request_id)
    .bind(tenant_uuid)
    .bind(store_uuid)
    .bind(cart_uuid)
    .bind(cart_item_id)
    .bind(variant_uuid)
    .bind(req.quantity)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::Cart {
        id: cart_uuid.to_string(),
        customer_id: String::new(),
        items: vec![pb::CartItem {
            id: cart_item_id.to_string(),
            vendor_id: vendor_id.map(|id| id.to_string()).unwrap_or_default(),
            variant_id: variant_uuid.to_string(),
            price: Some(pb::Money {
                amount: price_amount,
                currency: price_currency.clone(),
            }),
            quantity: req.quantity,
        }],
        total: Some(pb::Money {
            amount: price_amount.saturating_mul(req.quantity as i64),
            currency: price_currency,
        }),
        status: "active".to_string(),
    })
}

pub async fn remove_cart_item(
    state: &AppState,
    tenant_id: String,
    req: pb::RemoveCartItemRequest,
) -> Result<pb::Cart, (StatusCode, Json<ConnectError>)> {
    let tenant_uuid = parse_uuid(&tenant_id, "tenant_id")?;
    let cart_item_uuid = parse_uuid(&req.cart_item_id, "cart_item_id")?;

    let mut tx = state.db.begin().await.map_err(db::error)?;

    let row = sqlx::query(
        r#"
        SELECT c.id::text as cart_id, ci.variant_id::text as variant_id, ci.quantity
        FROM cart_items ci
        JOIN carts c ON c.id = ci.cart_id
        WHERE ci.id = $1 AND c.tenant_id = $2
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(cart_item_uuid)
    .bind(tenant_uuid)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::error)?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: "not_found",
                message: "cart item not found".to_string(),
            }),
        ));
    };

    let cart_id: String = row.get("cart_id");
    let variant_id: String = row.get("variant_id");
    let quantity: i32 = row.get("quantity");

    sqlx::query(
        r#"
        UPDATE inventory_reservations
        SET status = 'released', updated_at = now()
        WHERE cart_item_id = $1 AND status = 'active'
        "#,
    )
    .bind(cart_item_uuid)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    sqlx::query(
        r#"
        UPDATE inventory_stocks
        SET reserved = GREATEST(reserved - $1, 0),
            updated_at = now()
        WHERE variant_id = $2
        "#,
    )
    .bind(quantity)
    .bind(parse_uuid(&variant_id, "variant_id")?)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    sqlx::query("DELETE FROM cart_items WHERE id = $1")
        .bind(cart_item_uuid)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;

    tx.commit().await.map_err(db::error)?;

    Ok(pb::Cart {
        id: cart_id,
        customer_id: String::new(),
        items: Vec::new(),
        total: None,
        status: "active".to_string(),
    })
}

pub async fn update_cart_item(
    state: &AppState,
    tenant_id: String,
    req: pb::UpdateCartItemRequest,
) -> Result<pb::Cart, (StatusCode, Json<ConnectError>)> {
    if req.quantity <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "quantity must be greater than 0".to_string(),
            }),
        ));
    }

    let tenant_uuid = parse_uuid(&tenant_id, "tenant_id")?;
    let cart_item_uuid = parse_uuid(&req.cart_item_id, "cart_item_id")?;

    let mut tx = state.db.begin().await.map_err(db::error)?;
    let row = sqlx::query(
        r#"
        SELECT c.id::text as cart_id, ci.variant_id::text as variant_id, ci.quantity
        FROM cart_items ci
        JOIN carts c ON c.id = ci.cart_id
        WHERE ci.id = $1 AND c.tenant_id = $2
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(cart_item_uuid)
    .bind(tenant_uuid)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::error)?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: "not_found",
                message: "cart item not found".to_string(),
            }),
        ));
    };

    let cart_id: String = row.get("cart_id");
    let variant_id: String = row.get("variant_id");
    let current_qty: i32 = row.get("quantity");

    if req.quantity == current_qty {
        tx.commit().await.map_err(db::error)?;
        return Ok(pb::Cart {
            id: cart_id,
            customer_id: String::new(),
            items: Vec::new(),
            total: None,
            status: "active".to_string(),
        });
    }

    sqlx::query("UPDATE cart_items SET quantity = $1 WHERE id = $2")
        .bind(req.quantity)
        .bind(cart_item_uuid)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;

    let delta = req.quantity - current_qty;
    if delta > 0 {
        // Enqueue additional reservation for the delta.
        let store_row = sqlx::query(
            "SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1",
        )
        .bind(tenant_uuid)
        .fetch_one(&mut *tx)
        .await
        .map_err(db::error)?;
        let store_id: String = store_row.get("id");
        let store_uuid = parse_uuid(&store_id, "store_id")?;

        let request_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO inventory_reservation_requests (
                id, tenant_id, store_id, cart_id, cart_item_id, variant_id, quantity, status, is_hot
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,'queued',false)
            "#,
        )
        .bind(request_id)
        .bind(tenant_uuid)
        .bind(store_uuid)
        .bind(parse_uuid(&cart_id, "cart_id")?)
        .bind(cart_item_uuid)
        .bind(parse_uuid(&variant_id, "variant_id")?)
        .bind(delta)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    } else {
        let release_qty = -delta;
        // Reduce active reservation quantity if present.
        let reservation = sqlx::query(
            r#"
            SELECT id::text as id, quantity
            FROM inventory_reservations
            WHERE cart_item_id = $1 AND status = 'active'
            LIMIT 1
            FOR UPDATE
            "#,
        )
        .bind(cart_item_uuid)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::error)?;

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
                .map_err(db::error)?;
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
                .map_err(db::error)?;
            }
        }

        sqlx::query(
            r#"
            UPDATE inventory_stocks
            SET reserved = GREATEST(reserved - $1, 0),
                updated_at = now()
            WHERE variant_id = $2
            "#,
        )
        .bind(release_qty)
        .bind(parse_uuid(&variant_id, "variant_id")?)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    }

    tx.commit().await.map_err(db::error)?;

    Ok(pb::Cart {
        id: cart_id,
        customer_id: String::new(),
        items: Vec::new(),
        total: None,
        status: "active".to_string(),
    })
}

pub async fn checkout(
    state: &AppState,
    tenant_id: String,
    req: pb::CheckoutRequest,
) -> Result<pb::Order, (StatusCode, Json<ConnectError>)> {
    let tenant_uuid = parse_uuid(&tenant_id, "tenant_id")?;
    let cart_uuid = parse_uuid(&req.cart_id, "cart_id")?;
    let payment_method = payment_method_to_string(req.payment_method).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "payment_method is required".to_string(),
            }),
        )
    })?;

    let mut tx = state.db.begin().await.map_err(db::error)?;
    let cart_row = sqlx::query(
        "SELECT customer_id::text as customer_id FROM carts WHERE id = $1 AND tenant_id = $2 LIMIT 1 FOR UPDATE",
    )
    .bind(cart_uuid)
    .bind(tenant_uuid)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::error)?;
    let Some(cart_row) = cart_row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: "not_found",
                message: "cart not found".to_string(),
            }),
        ));
    };

    let items = sqlx::query(
        r#"
        SELECT id::text as cart_item_id, variant_id::text as variant_id,
               vendor_id::text as vendor_id, price_amount, price_currency, quantity
        FROM cart_items
        WHERE cart_id = $1
        FOR UPDATE
        "#,
    )
    .bind(cart_uuid)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::error)?;

    if items.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "cart has no items".to_string(),
            }),
        ));
    }

    let mut total_amount: i64 = 0;
    let mut currency: Option<String> = None;

    for item in &items {
        let cart_item_id: String = item.get("cart_item_id");
        let variant_id: String = item.get("variant_id");
        let quantity: i32 = item.get("quantity");
        let price_amount: i64 = item.get("price_amount");
        let price_currency: String = item.get("price_currency");

        if let Some(curr) = &currency {
            if curr != &price_currency {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ConnectError {
                        code: "invalid_argument",
                        message: "mixed currency cart is not supported".to_string(),
                    }),
                ));
            }
        } else {
            currency = Some(price_currency.clone());
        }

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
        .map_err(db::error)?;

        let Some(reservation) = reservation else {
            return Err((
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: "failed_precondition",
                    message: "inventory reservation not ready".to_string(),
                }),
            ));
        };
        let reserved_qty: i32 = reservation.get("quantity");
        if reserved_qty < quantity {
            return Err((
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: "failed_precondition",
                    message: "reserved quantity is insufficient".to_string(),
                }),
            ));
        }

        let updated = sqlx::query(
            r#"
            UPDATE inventory_stocks
            SET stock = stock - $1,
                reserved = reserved - $1,
                updated_at = now()
            WHERE variant_id = $2
              AND stock >= $1
              AND reserved >= $1
            "#,
        )
        .bind(quantity)
        .bind(parse_uuid(&variant_id, "variant_id")?)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
        if updated.rows_affected() != 1 {
            return Err((
                StatusCode::CONFLICT,
                Json(ConnectError {
                    code: "failed_precondition",
                    message: "inventory stock is insufficient".to_string(),
                }),
            ));
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
        .map_err(db::error)?;

        total_amount = total_amount.saturating_add(price_amount * (quantity as i64));
    }

    let order_id = uuid::Uuid::new_v4();
    let status = match payment_method {
        "bank_transfer" => "pending_payment",
        "cod" => "pending_shipment",
        _ => "pending_payment",
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
    .bind(payment_method)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    for item in &items {
        let cart_item_id: String = item.get("cart_item_id");
        let variant_id: String = item.get("variant_id");
        let vendor_id: Option<String> = item.try_get("vendor_id").ok();
        let quantity: i32 = item.get("quantity");
        let price_amount: i64 = item.get("price_amount");
        let price_currency: String = item.get("price_currency");

        sqlx::query(
            r#"
            INSERT INTO order_items (id, order_id, vendor_id, variant_id, price_amount, price_currency, quantity)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(order_id)
    .bind(vendor_id.as_deref().and_then(|id| parse_uuid(id, "vendor_id").ok()))
        .bind(parse_uuid(&variant_id, "variant_id")?)
        .bind(price_amount)
        .bind(price_currency)
        .bind(quantity)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;

        sqlx::query("DELETE FROM cart_items WHERE id = $1")
            .bind(parse_uuid(&cart_item_id, "cart_item_id")?)
            .execute(&mut *tx)
            .await
            .map_err(db::error)?;
    }

    sqlx::query("UPDATE carts SET status = 'ordered', updated_at = now() WHERE id = $1")
        .bind(cart_uuid)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;

    tx.commit().await.map_err(db::error)?;

    Ok(pb::Order {
        id: order_id.to_string(),
        customer_id: cart_row.get::<Option<String>, _>("customer_id").unwrap_or_default(),
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
