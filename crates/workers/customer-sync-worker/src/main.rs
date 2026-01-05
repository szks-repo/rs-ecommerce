use std::time::Duration;

use anyhow::Result;
use rs_common::{env, telemetry};
use serde::Deserialize;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use tracing::{info, warn};

#[derive(Debug)]
struct OutboxEvent {
    id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    store_id: Option<uuid::Uuid>,
    event_type: String,
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ProfilePayload {
    tenant_id: String,
    source_store_id: Option<String>,
    customer_id: String,
    profile: ProfileData,
}

#[derive(Debug, Deserialize)]
struct ProfileData {
    name: String,
    email: String,
    phone: String,
    status: String,
    notes: String,
}

#[derive(Debug, Deserialize)]
struct IdentityPayload {
    tenant_id: String,
    source_store_id: Option<String>,
    customer_id: String,
    identity: IdentityData,
}

#[derive(Debug, Deserialize)]
struct IdentityData {
    identity_type: String,
    identity_value: String,
    verified: bool,
    source: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_tracing("customer-sync-worker");
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await?;

    wait_for_schema(&pool).await?;

    let batch_size = env::env_usize("CUSTOMER_SYNC_BATCH_SIZE", 100) as i64;
    let sleep_ms = env::env_u64("CUSTOMER_SYNC_WORKER_SLEEP_MS", 1000);
    let oneshot = env::env_bool("CUSTOMER_SYNC_WORKER_ONESHOT", false);

    loop {
        let processed = process_outbox_batch(&pool, batch_size).await?;
        info!(processed, "customer sync batch processed");
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
        let row = sqlx::query("SELECT to_regclass('public.outbox_events')::text as outbox_table")
            .fetch_one(pool)
            .await?;
        let exists: Option<String> = row.get("outbox_table");
        if exists.is_some() {
            return Ok(());
        }
        attempts += 1;
        if attempts % 10 == 0 {
            info!(attempts, "waiting for migrations to create outbox tables");
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn process_outbox_batch(pool: &PgPool, batch_size: i64) -> Result<usize> {
    let mut tx = pool.begin().await?;
    let rows = sqlx::query(
        r#"
        WITH cte AS (
            SELECT id
            FROM outbox_events
            WHERE status = 'pending'
              AND event_type IN ('customer.profile_upsert', 'customer.identity_upsert', 'customer.address_upsert')
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE outbox_events AS o
        SET status = 'processing'
        FROM cte
        WHERE o.id = cte.id
        RETURNING o.id, o.tenant_id, o.store_id, o.event_type, o.payload_json
        "#,
    )
    .bind(batch_size)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    let mut processed = 0usize;
    for row in rows {
        let event = OutboxEvent {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            store_id: row.get("store_id"),
            event_type: row.get("event_type"),
            payload: row.get("payload_json"),
        };
        match handle_event(pool, &event).await {
            Ok(()) => {
                processed += 1;
                sqlx::query("UPDATE outbox_events SET status = 'published', published_at = now() WHERE id = $1")
                    .bind(event.id)
                    .execute(pool)
                    .await?;
            }
            Err(err) => {
                warn!(error = %err, event_id = %event.id, "customer sync event failed");
                sqlx::query("UPDATE outbox_events SET status = 'failed' WHERE id = $1")
                    .bind(event.id)
                    .execute(pool)
                    .await?;
            }
        }
    }

    Ok(processed)
}

async fn handle_event(pool: &PgPool, event: &OutboxEvent) -> Result<()> {
    match event.event_type.as_str() {
        "customer.profile_upsert" => {
            let payload: ProfilePayload = serde_json::from_value(event.payload.clone())?;
            apply_profile_sync(pool, event.id, payload).await?;
        }
        "customer.identity_upsert" => {
            let payload: IdentityPayload = serde_json::from_value(event.payload.clone())?;
            apply_identity_sync(pool, event.id, payload).await?;
        }
        "customer.address_upsert" => {
            // Currently not fanned-out across stores.
        }
        _ => {}
    }
    Ok(())
}

async fn apply_profile_sync(pool: &PgPool, event_id: uuid::Uuid, payload: ProfilePayload) -> Result<()> {
    let tenant_id = uuid::Uuid::parse_str(&payload.tenant_id)?;
    let customer_id = uuid::Uuid::parse_str(&payload.customer_id)?;
    let source_store_id = payload.source_store_id.and_then(|s| uuid::Uuid::parse_str(&s).ok());
    let name = payload.profile.name.clone();
    let email = payload.profile.email.clone();
    let phone = payload.profile.phone.clone();
    let status = payload.profile.status.clone();
    let notes = payload.profile.notes.clone();

    let target_rows = sqlx::query(
        r#"
        SELECT store_id::text as store_id
        FROM store_sync_settings
        WHERE tenant_id = $1 AND customer_sync_enabled = true
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    for row in target_rows {
        let store_id_str: String = row.get("store_id");
        let store_id = uuid::Uuid::parse_str(&store_id_str)?;
        if Some(store_id) == source_store_id {
            continue;
        }
        if already_processed(pool, tenant_id, event_id, store_id).await? {
            continue;
        }

        sqlx::query(
            r#"
            INSERT INTO customers (id, tenant_id, status)
            VALUES ($1,$2,'active')
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(customer_id)
        .bind(tenant_id)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO customer_profiles (id, customer_id, store_id, name, email, phone, status, notes)
            VALUES (gen_random_uuid(), $1,$2,$3,$4,$5,$6,$7)
            ON CONFLICT (customer_id, store_id)
            DO UPDATE SET
                name = CASE WHEN customer_profiles.name = '' THEN EXCLUDED.name ELSE customer_profiles.name END,
                email = CASE WHEN customer_profiles.email IS NULL OR customer_profiles.email = '' THEN EXCLUDED.email ELSE customer_profiles.email END,
                phone = CASE WHEN customer_profiles.phone IS NULL OR customer_profiles.phone = '' THEN EXCLUDED.phone ELSE customer_profiles.phone END,
                status = CASE WHEN customer_profiles.status = '' THEN EXCLUDED.status ELSE customer_profiles.status END,
                updated_at = now()
            "#,
        )
        .bind(customer_id)
        .bind(store_id)
        .bind(&name)
        .bind(if email.is_empty() { None } else { Some(email.clone()) })
        .bind(if phone.is_empty() { None } else { Some(phone.clone()) })
        .bind(if status.is_empty() { "active" } else { status.as_str() })
        .bind(if notes.is_empty() { None } else { Some(notes.clone()) })
        .execute(pool)
        .await?;

        mark_processed(pool, tenant_id, event_id, store_id).await?;
    }

    Ok(())
}

async fn apply_identity_sync(pool: &PgPool, event_id: uuid::Uuid, payload: IdentityPayload) -> Result<()> {
    let tenant_id = uuid::Uuid::parse_str(&payload.tenant_id)?;
    let customer_id = uuid::Uuid::parse_str(&payload.customer_id)?;
    let identity_value = normalize_identity(&payload.identity.identity_type, &payload.identity.identity_value);

    sqlx::query(
        r#"
        INSERT INTO customer_identities
            (id, tenant_id, customer_id, identity_type, identity_value, verified, source)
        VALUES (gen_random_uuid(), $1,$2,$3,$4,$5,$6)
        ON CONFLICT (tenant_id, identity_type, identity_value)
        DO UPDATE SET verified = EXCLUDED.verified,
                      source = EXCLUDED.source
        WHERE customer_id = EXCLUDED.customer_id
        "#,
    )
    .bind(tenant_id)
    .bind(customer_id)
    .bind(&payload.identity.identity_type)
    .bind(identity_value)
    .bind(payload.identity.verified)
    .bind(if payload.identity.source.is_empty() {
        "admin"
    } else {
        payload.identity.source.as_str()
    })
    .execute(pool)
    .await?;

    if let Some(source_store_id) = payload.source_store_id.and_then(|s| uuid::Uuid::parse_str(&s).ok()) {
        mark_processed(pool, tenant_id, event_id, source_store_id).await?;
    }
    Ok(())
}

async fn already_processed(
    pool: &PgPool,
    tenant_id: uuid::Uuid,
    event_id: uuid::Uuid,
    store_id: uuid::Uuid,
) -> Result<bool> {
    let row = sqlx::query("SELECT 1 FROM processed_events WHERE tenant_id = $1 AND event_id = $2 AND store_id = $3")
        .bind(tenant_id)
        .bind(event_id)
        .bind(store_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

async fn mark_processed(
    pool: &PgPool,
    tenant_id: uuid::Uuid,
    event_id: uuid::Uuid,
    store_id: uuid::Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO processed_events (tenant_id, store_id, event_id)
        VALUES ($1,$2,$3)
        ON CONFLICT (tenant_id, event_id, store_id) DO NOTHING
        "#,
    )
    .bind(tenant_id)
    .bind(store_id)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

fn normalize_identity(identity_type: &str, value: &str) -> String {
    let trimmed = value.trim();
    match identity_type {
        "email" => trimmed.to_lowercase(),
        "phone" => trimmed.chars().filter(|c| c.is_ascii_digit()).collect(),
        _ => trimmed.to_string(),
    }
}
