use axum::{Json, http::StatusCode};

use crate::{
    AppState,
    pb::pb,
    infrastructure::audit,
    rpc::json::ConnectError,
    shared::{
        audit_action::{ShippingRateAuditAction, ShippingZoneAuditAction},
        ids::{parse_uuid, StoreId, TenantId},
        money::{money_from_parts, money_to_parts},
    },
    store_settings::repository::{PgStoreSettingsRepository, StoreSettingsRepository},
};

use crate::shared::audit_helpers::{audit_input, to_json_opt};

pub async fn list_shipping_zones(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::ShippingZone>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let zones = repo.list_shipping_zones(&store_uuid.as_uuid()).await?;
    let mut result = Vec::new();
    for zone in zones {
        let zone_uuid = parse_uuid(&zone.id, "zone_id")?;
        let prefs = repo.list_zone_prefectures(&zone_uuid).await?;
        let prefectures = prefs
            .into_iter()
            .map(|row| pb::Prefecture {
                code: row.code,
                name: row.name,
            })
            .collect();
        result.push(pb::ShippingZone {
            id: zone.id,
            name: zone.name,
            domestic_only: zone.domestic_only,
            prefectures,
        });
    }
    Ok(result)
}

pub async fn upsert_shipping_zone(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    zone: pb::ShippingZone,
    actor: Option<pb::ActorContext>,
) -> Result<pb::ShippingZone, (StatusCode, Json<ConnectError>)> {
    validate_shipping_zone(&zone)?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let zone_id = if zone.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&zone.id, "zone_id")?
    };

    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(crate::infrastructure::db::error)?;
    if zone.id.is_empty() {
        repo.insert_shipping_zone_tx(
            &mut tx,
            &zone_id,
            &store_uuid.as_uuid(),
            &tenant_uuid.as_uuid(),
            &zone,
        )
        .await?;
    } else {
        repo.update_shipping_zone_tx(&mut tx, &zone_id, &store_uuid.as_uuid(), &zone)
            .await?;
        repo.delete_zone_prefectures_tx(&mut tx, &zone_id).await?;
    }

    for pref in &zone.prefectures {
        repo.insert_zone_prefecture_tx(&mut tx, &zone_id, pref).await?;
    }

    tx.commit().await.map_err(crate::infrastructure::db::error)?;

    let updated = pb::ShippingZone {
        id: zone_id.to_string(),
        name: zone.name,
        domestic_only: zone.domestic_only,
        prefectures: zone.prefectures,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            ShippingZoneAuditAction::Upsert.into(),
            Some("shipping_zone"),
            Some(updated.id.clone()),
            None,
            to_json_opt(Some(updated.clone())),
            actor.clone(),
        ),
    )
    .await?;

    Ok(updated)
}

pub async fn delete_shipping_zone(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    zone_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    let zone_uuid = parse_uuid(&zone_id, "zone_id")?;
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(crate::infrastructure::db::error)?;
    repo.delete_zone_prefectures_tx(&mut tx, &zone_uuid).await?;
    let rows = repo
        .delete_shipping_zone_tx(&mut tx, &zone_uuid, &store_uuid.as_uuid())
        .await?;
    tx.commit().await.map_err(crate::infrastructure::db::error)?;
    let deleted = rows > 0;
    if deleted {
        let _ = audit::record(
            state,
            audit_input(
                tenant_id.clone(),
                ShippingZoneAuditAction::Delete.into(),
                Some("shipping_zone"),
                Some(zone_id),
                None,
                None,
                actor.clone(),
            ),
        )
        .await?;
    }
    Ok(deleted)
}

pub async fn list_shipping_rates(
    state: &AppState,
    store_id: String,
    zone_id: String,
) -> Result<Vec<pb::ShippingRate>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let zone_uuid = parse_uuid(&zone_id, "zone_id")?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let rows = repo
        .list_shipping_rates(&store_uuid.as_uuid(), &zone_uuid)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| pb::ShippingRate {
            id: row.id,
            zone_id: row.zone_id,
            name: row.name,
            min_subtotal: row
                .min_subtotal_amount
                .zip(Some(row.fee_currency.clone()))
                .map(|(amount, currency)| money_from_parts(amount, currency)),
            max_subtotal: row
                .max_subtotal_amount
                .zip(Some(row.fee_currency.clone()))
                .map(|(amount, currency)| money_from_parts(amount, currency)),
            fee: Some(money_from_parts(row.fee_amount, row.fee_currency)),
        })
        .collect())
}

pub async fn upsert_shipping_rate(
    state: &AppState,
    _store_id: String,
    tenant_id: String,
    rate: pb::ShippingRate,
    actor: Option<pb::ActorContext>,
) -> Result<pb::ShippingRate, (StatusCode, Json<ConnectError>)> {
    validate_shipping_rate(&rate)?;
    let rate_id = if rate.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&rate.id, "rate_id")?
    };
    let (fee_amount, fee_currency) = money_to_parts(rate.fee.clone())?;
    let min = rate.min_subtotal.clone().map(|m| m.amount);
    let max = rate.max_subtotal.clone().map(|m| m.amount);
    let repo = PgStoreSettingsRepository::new(&state.db);
    if rate.id.is_empty() {
        let zone_uuid = parse_uuid(&rate.zone_id, "zone_id")?;
        repo.insert_shipping_rate(
            &rate_id,
            &zone_uuid,
            &rate,
            fee_amount,
            &fee_currency,
            min,
            max,
        )
        .await?;
    } else {
        repo.update_shipping_rate(&rate_id, &rate, fee_amount, &fee_currency, min, max)
            .await?;
    }

    let _ = tenant_id;
    let updated = pb::ShippingRate {
        id: rate_id.to_string(),
        zone_id: rate.zone_id,
        name: rate.name,
        min_subtotal: rate.min_subtotal,
        max_subtotal: rate.max_subtotal,
        fee: rate.fee,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            ShippingRateAuditAction::Upsert.into(),
            Some("shipping_rate"),
            Some(updated.id.clone()),
            None,
            to_json_opt(Some(updated.clone())),
            actor.clone(),
        ),
    )
    .await?;

    Ok(updated)
}

pub async fn delete_shipping_rate(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    rate_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let rows = repo
        .delete_shipping_rate(
            &store_uuid.as_uuid(),
            &parse_uuid(&rate_id, "rate_id")?,
        )
        .await?;
    let deleted = rows > 0;
    if deleted {
        let _ = audit::record(
            state,
            audit_input(
                tenant_id.clone(),
                ShippingRateAuditAction::Delete.into(),
                Some("shipping_rate"),
                Some(rate_id),
                None,
                None,
                actor.clone(),
            ),
        )
        .await?;
    }
    Ok(deleted)
}

pub fn validate_shipping_rate(
    rate: &pb::ShippingRate,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if let (Some(min), Some(max)) = (&rate.min_subtotal, &rate.max_subtotal) {
        if min.amount > max.amount {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "min_subtotal must be <= max_subtotal".to_string(),
                }),
            ));
        }
    }
    if let Some(fee) = &rate.fee {
        if fee.amount < 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "fee must be >= 0".to_string(),
                }),
            ));
        }
    }
    Ok(())
}

pub fn validate_shipping_zone(
    zone: &pb::ShippingZone,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    for pref in &zone.prefectures {
        if !is_valid_prefecture_code(&pref.code) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: format!("invalid prefecture code: {}", pref.code),
                }),
            ));
        }
    }
    Ok(())
}

fn is_valid_prefecture_code(code: &str) -> bool {
    if !code.starts_with("JP-") || code.len() != 5 {
        return false;
    }
    let num = &code[3..5];
    match num.parse::<u8>() {
        Ok(n) => (1..=47).contains(&n),
        Err(_) => false,
    }
}
