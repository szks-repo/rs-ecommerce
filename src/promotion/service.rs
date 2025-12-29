use axum::{Json, http::StatusCode};
use sqlx::Row;
use serde_json::Value;

use crate::{
    AppState,
    pb::pb,
    infrastructure::{db, audit},
    rpc::json::ConnectError,
    shared::{audit_action::{AuditAction, PromotionAuditAction}, money::money_to_parts, time::timestamp_to_chrono},
};

pub async fn create_promotion(
    state: &AppState,
    tenant_id: String,
    req: pb::CreatePromotionRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::PromotionAdmin, (StatusCode, Json<ConnectError>)> {
    let (value_amount, value_currency) = money_to_parts(req.value.clone())?;
    let promotion_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO promotions (
            id, tenant_id, code, discount_type, value_amount, value_currency,
            status, starts_at, ends_at
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        "#,
    )
    .bind(promotion_id)
    .bind(&tenant_id)
    .bind(&req.code)
    .bind(&req.discount_type)
    .bind(value_amount)
    .bind(&value_currency)
    .bind(&req.status)
    .bind(timestamp_to_chrono(req.starts_at.clone()))
    .bind(timestamp_to_chrono(req.ends_at.clone()))
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let promotion = pb::PromotionAdmin {
        id: promotion_id.to_string(),
        code: req.code,
        discount_type: req.discount_type,
        value: req.value,
        status: req.status,
        starts_at: req.starts_at,
        ends_at: req.ends_at,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            PromotionAuditAction::Create.into(),
            Some("promotion"),
            Some(promotion.id.clone()),
            None,
            to_json_opt(Some(promotion.clone())),
            _actor,
        ),
    )
    .await?;

    Ok(promotion)
}

pub async fn update_promotion(
    state: &AppState,
    tenant_id: String,
    req: pb::UpdatePromotionRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::PromotionAdmin, (StatusCode, Json<ConnectError>)> {
    let before = fetch_promotion(state, &tenant_id, &req.promotion_id).await.ok();
    let (value_amount, value_currency) = money_to_parts(req.value.clone())?;
    sqlx::query(
        r#"
        UPDATE promotions
        SET code = $1, discount_type = $2, value_amount = $3, value_currency = $4,
            status = $5, starts_at = $6, ends_at = $7
        WHERE id = $8 AND tenant_id = $9
        "#,
    )
    .bind(&req.code)
    .bind(&req.discount_type)
    .bind(value_amount)
    .bind(&value_currency)
    .bind(&req.status)
    .bind(timestamp_to_chrono(req.starts_at.clone()))
    .bind(timestamp_to_chrono(req.ends_at.clone()))
    .bind(crate::shared::ids::parse_uuid(&req.promotion_id, "promotion_id")?)
    .bind(&tenant_id)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let promotion = pb::PromotionAdmin {
        id: req.promotion_id,
        code: req.code,
        discount_type: req.discount_type,
        value: req.value,
        status: req.status,
        starts_at: req.starts_at,
        ends_at: req.ends_at,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            PromotionAuditAction::Update.into(),
            Some("promotion"),
            Some(promotion.id.clone()),
            to_json_opt(before),
            to_json_opt(Some(promotion.clone())),
            _actor,
        ),
    )
    .await?;

    Ok(promotion)
}

fn audit_input(
    tenant_id: String,
    action: AuditAction,
    target_type: Option<&str>,
    target_id: Option<String>,
    before_json: Option<Value>,
    after_json: Option<Value>,
    actor: Option<pb::ActorContext>,
) -> audit::AuditInput {
    let (actor_id, actor_type) = actor_fields(actor);
    audit::AuditInput {
        tenant_id,
        actor_id,
        actor_type,
        action,
        target_type: target_type.map(|v| v.to_string()),
        target_id,
        request_id: None,
        ip_address: None,
        user_agent: None,
        before_json,
        after_json,
        metadata_json: None,
    }
}

fn actor_fields(actor: Option<pb::ActorContext>) -> (Option<String>, String) {
    let actor_id = actor
        .as_ref()
        .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
    let actor_type = actor
        .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type) })
        .unwrap_or_else(|| "system".to_string());
    (actor_id, actor_type)
}

fn to_json_opt<T: serde::Serialize>(value: Option<T>) -> Option<Value> {
    value.and_then(|v| serde_json::to_value(v).ok())
}

async fn fetch_promotion(
    state: &AppState,
    tenant_id: &str,
    promotion_id: &str,
) -> Result<pb::PromotionAdmin, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT id::text as id, code, discount_type, value_amount, value_currency,
               status, starts_at, ends_at
        FROM promotions
        WHERE tenant_id = $1 AND id = $2
        "#,
    )
    .bind(tenant_id)
    .bind(crate::shared::ids::parse_uuid(promotion_id, "promotion_id")?)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::PromotionAdmin {
        id: row.get("id"),
        code: row.get("code"),
        discount_type: row.get("discount_type"),
        value: Some(pb::Money {
            amount: row.get::<i64, _>("value_amount"),
            currency: row.get::<String, _>("value_currency"),
        }),
        status: row.get("status"),
        starts_at: crate::shared::time::chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("starts_at")),
        ends_at: crate::shared::time::chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("ends_at")),
    })
}
