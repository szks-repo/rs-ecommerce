use std::time::Duration;

use anyhow::Result;
use rs_common::{env, telemetry};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use tracing::info;

#[derive(Debug)]
struct ReservationRequest {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    cart_id: uuid::Uuid,
    cart_item_id: Option<uuid::Uuid>,
    sku_id: uuid::Uuid,
    location_id: Option<uuid::Uuid>,
    quantity: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_tracing("inventory-worker");
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await?;

    wait_for_schema(&pool).await?;

    let batch_size = env::env_usize("INVENTORY_WORKER_BATCH_SIZE", 50) as i64;
    let ttl_seconds = env::env_i64("INVENTORY_RESERVATION_TTL_SECONDS", 900);
    let sleep_ms = env::env_u64("INVENTORY_WORKER_SLEEP_MS", 500);
    let oneshot = env::env_bool("INVENTORY_WORKER_ONESHOT", false);

    loop {
        let (hot_done, hot_failed) = process_queue_batch(&pool, batch_size, ttl_seconds, true).await?;
        let (normal_done, normal_failed) = process_queue_batch(&pool, batch_size, ttl_seconds, false).await?;
        let released = release_expired_reservations(&pool, batch_size).await?;

        info!(
            hot_done,
            hot_failed, normal_done, normal_failed, released, "inventory worker batch completed"
        );

        if oneshot {
            break;
        }
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
    }

    Ok(())
}

async fn wait_for_schema(pool: &PgPool) -> Result<()> {
    let mut attempts = 0;
    loop {
        let row = sqlx::query("SELECT to_regclass('public.inventory_reservation_requests')::text as req_table")
            .fetch_one(pool)
            .await?;
        let exists: Option<String> = row.get("req_table");
        if exists.is_some() {
            return Ok(());
        }
        attempts += 1;
        if attempts % 10 == 0 {
            info!(
                attempts,
                "waiting for migrations to create inventory reservation tables"
            );
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn process_queue_batch(pool: &PgPool, batch_size: i64, ttl_seconds: i64, is_hot: bool) -> Result<(usize, usize)> {
    let mut tx = pool.begin().await?;
    let mut done = 0usize;
    let mut failed = 0usize;
    let rows = sqlx::query(
        r#"
        WITH cte AS (
            SELECT id
            FROM inventory_reservation_requests
            WHERE status = 'queued' AND is_hot = $1
            ORDER BY created_at ASC
            LIMIT $2
            FOR UPDATE SKIP LOCKED
        )
        UPDATE inventory_reservation_requests AS r
        SET status = 'processing', updated_at = now()
        FROM cte
        WHERE r.id = cte.id
        RETURNING r.id, r.store_id, r.cart_id, r.cart_item_id, r.sku_id, r.location_id, r.quantity
        "#,
    )
    .bind(is_hot)
    .bind(batch_size)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    for row in rows {
        let request = ReservationRequest {
            id: row.get("id"),
            store_id: row.get("store_id"),
            cart_id: row.get("cart_id"),
            cart_item_id: row.get("cart_item_id"),
            sku_id: row.get("sku_id"),
            location_id: row.get("location_id"),
            quantity: row.get("quantity"),
        };
        if process_request(pool, request, ttl_seconds).await? {
            done += 1;
        } else {
            failed += 1;
        }
    }

    Ok((done, failed))
}

async fn process_request(pool: &PgPool, request: ReservationRequest, ttl_seconds: i64) -> Result<bool> {
    let Some(location_id) = request.location_id else {
        let mut tx = pool.begin().await?;
        sqlx::query(
            r#"
            UPDATE inventory_reservation_requests
            SET status = 'failed', updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(request.id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(false);
    };

    let mut tx = pool.begin().await?;
    let stock_row = sqlx::query(
        r#"
        SELECT on_hand, reserved
        FROM inventory_stocks
        WHERE sku_id = $1
          AND location_id = $2
          AND store_id = $3
        FOR UPDATE
        "#,
    )
    .bind(request.sku_id)
    .bind(location_id)
    .bind(request.store_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(stock_row) = stock_row else {
        sqlx::query(
            r#"
            UPDATE inventory_reservation_requests
            SET status = 'failed', updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(request.id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(false);
    };

    let before_on_hand: i32 = stock_row.get("on_hand");
    let before_reserved: i32 = stock_row.get("reserved");
    if before_on_hand - before_reserved < request.quantity {
        sqlx::query(
            r#"
            UPDATE inventory_reservation_requests
            SET status = 'failed', updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(request.id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(false);
    }

    let after_reserved = before_reserved + request.quantity;
    sqlx::query(
        r#"
        UPDATE inventory_stocks
        SET reserved = $1,
            updated_at = now()
        WHERE sku_id = $2
          AND location_id = $3
          AND store_id = $4
        "#,
    )
    .bind(after_reserved)
    .bind(request.sku_id)
    .bind(location_id)
    .bind(request.store_id)
    .execute(&mut *tx)
    .await?;

    insert_inventory_movement_tx(
        &mut tx,
        request.store_id,
        request.sku_id,
        location_id,
        "reserve",
        request.quantity,
        before_on_hand,
        before_on_hand,
        before_reserved,
        after_reserved,
        Some("cart_reservation"),
        Some(request.cart_id),
    )
    .await?;

    {
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds);
        sqlx::query(
            r#"
            INSERT INTO inventory_reservations (
                id, store_id, cart_id, cart_item_id, sku_id, location_id,
                quantity, status, expires_at, created_at, updated_at
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,'active',$8,now(),now())
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(request.store_id)
        .bind(request.cart_id)
        .bind(request.cart_item_id)
        .bind(request.sku_id)
        .bind(location_id)
        .bind(request.quantity)
        .bind(expires_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE inventory_reservation_requests
            SET status = 'done', updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(request.id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(true)
}

async fn release_expired_reservations(pool: &PgPool, batch_size: i64) -> Result<usize> {
    let mut tx = pool.begin().await?;
    let rows = sqlx::query(
        r#"
        WITH cte AS (
            SELECT id, store_id, sku_id, location_id, quantity
            FROM inventory_reservations
            WHERE status = 'active' AND expires_at < now()
            ORDER BY expires_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE inventory_reservations AS r
        SET status = 'expired', updated_at = now()
        FROM cte
        WHERE r.id = cte.id
        RETURNING cte.store_id, cte.sku_id, cte.location_id, cte.quantity
        "#,
    )
    .bind(batch_size)
    .fetch_all(&mut *tx)
    .await?;

    let released = rows.len();
    for row in rows {
        let store_id: uuid::Uuid = row.get("store_id");
        let sku_id: uuid::Uuid = row.get("sku_id");
        let location_id: Option<uuid::Uuid> = row.get("location_id");
        let quantity: i32 = row.get("quantity");
        if let Some(location_id) = location_id {
            let stock_row = sqlx::query(
                r#"
                SELECT on_hand, reserved
                FROM inventory_stocks
                WHERE sku_id = $1
                  AND location_id = $2
                  AND store_id = $3
                FOR UPDATE
                "#,
            )
            .bind(sku_id)
            .bind(location_id)
            .bind(store_id)
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(stock_row) = stock_row {
                let before_on_hand: i32 = stock_row.get("on_hand");
                let before_reserved: i32 = stock_row.get("reserved");
                let after_reserved = (before_reserved - quantity).max(0);

                sqlx::query(
                    r#"
                    UPDATE inventory_stocks
                    SET reserved = $1,
                        updated_at = now()
                    WHERE sku_id = $2
                      AND location_id = $3
                      AND store_id = $4
                    "#,
                )
                .bind(after_reserved)
                .bind(sku_id)
                .bind(location_id)
                .bind(store_id)
                .execute(&mut *tx)
                .await?;

                insert_inventory_movement_tx(
                    &mut tx,
                    store_id,
                    sku_id,
                    location_id,
                    "release",
                    quantity,
                    before_on_hand,
                    before_on_hand,
                    before_reserved,
                    after_reserved,
                    Some("reservation_expired"),
                    None,
                )
                .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(released)
}

async fn insert_inventory_movement_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: uuid::Uuid,
    sku_id: uuid::Uuid,
    location_id: uuid::Uuid,
    movement_type: &str,
    quantity: i32,
    before_on_hand: i32,
    after_on_hand: i32,
    before_reserved: i32,
    after_reserved: i32,
    source_type: Option<&str>,
    source_id: Option<uuid::Uuid>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO inventory_movements (
            store_id, sku_id, location_id, movement_type, quantity,
            before_on_hand, after_on_hand, before_reserved, after_reserved,
            source_type, source_id
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        "#,
    )
    .bind(store_id)
    .bind(sku_id)
    .bind(location_id)
    .bind(movement_type)
    .bind(quantity)
    .bind(before_on_hand)
    .bind(after_on_hand)
    .bind(before_reserved)
    .bind(after_reserved)
    .bind(source_type)
    .bind(source_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
