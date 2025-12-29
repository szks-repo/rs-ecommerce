use axum::{Json, http::StatusCode};
use sqlx::Row;

use crate::{
    AppState,
    pb::pb,
    infrastructure::{db, audit},
    rpc::json::ConnectError,
    shared::{
        audit_action::{
            AuditAction,
            StoreSettingsAuditAction,
            MallSettingsAuditAction,
            StoreLocationAuditAction,
            ShippingZoneAuditAction,
            ShippingRateAuditAction,
            TaxRuleAuditAction,
        },
        ids::{parse_uuid, StoreId, TenantId},
        money::{money_from_parts, money_to_parts},
    },
};
use crate::rpc::request_context;

pub async fn get_store_settings(
    state: &AppState,
    store_id: String,
) -> Result<pb::StoreSettings, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let row = sqlx::query(
        r#"
        SELECT store_name, legal_name, contact_email, contact_phone,
               address_prefecture, address_city, address_line1, address_line2,
               legal_notice, default_language, primary_domain, subdomain, https_enabled,
               currency, tax_mode, tax_rounding, order_initial_status,
               cod_enabled, cod_fee_amount, cod_fee_currency,
               bank_name, bank_branch, bank_account_type, bank_account_number, bank_account_name,
               theme, brand_color, logo_url, favicon_url
        FROM store_settings
        WHERE store_id = $1
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::StoreSettings {
        store_name: row.get("store_name"),
        legal_name: row.get("legal_name"),
        contact_email: row.get("contact_email"),
        contact_phone: row.get("contact_phone"),
        address_prefecture: row.get("address_prefecture"),
        address_city: row.get("address_city"),
        address_line1: row.get("address_line1"),
        address_line2: row.get::<Option<String>, _>("address_line2").unwrap_or_default(),
        legal_notice: row.get("legal_notice"),
        default_language: row.get("default_language"),
        primary_domain: row.get::<Option<String>, _>("primary_domain").unwrap_or_default(),
        subdomain: row.get::<Option<String>, _>("subdomain").unwrap_or_default(),
        https_enabled: row.get("https_enabled"),
        currency: row.get("currency"),
        tax_mode: row.get("tax_mode"),
        tax_rounding: row.get("tax_rounding"),
        order_initial_status: row.get("order_initial_status"),
        cod_enabled: row.get("cod_enabled"),
        cod_fee: Some(money_from_parts(
            row.get::<i64, _>("cod_fee_amount"),
            row.get::<String, _>("cod_fee_currency"),
        )),
        bank_name: row.get("bank_name"),
        bank_branch: row.get("bank_branch"),
        bank_account_type: row.get("bank_account_type"),
        bank_account_number: row.get("bank_account_number"),
        bank_account_name: row.get("bank_account_name"),
        theme: row.get("theme"),
        brand_color: row.get("brand_color"),
        logo_url: row.get::<Option<String>, _>("logo_url").unwrap_or_default(),
        favicon_url: row.get::<Option<String>, _>("favicon_url").unwrap_or_default(),
    })
}

pub async fn update_store_settings(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    settings: pb::StoreSettings,
    actor: Option<pb::ActorContext>,
) -> Result<pb::StoreSettings, (StatusCode, Json<ConnectError>)> {
    validate_store_settings(&settings)?;
    let before = get_store_settings(state, store_id.clone()).await.ok();
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    let (cod_fee_amount, cod_fee_currency) = money_to_parts(settings.cod_fee.clone())?;
    sqlx::query(
        r#"
        UPDATE store_settings
        SET store_name = $1, legal_name = $2, contact_email = $3, contact_phone = $4,
            address_prefecture = $5, address_city = $6, address_line1 = $7, address_line2 = $8,
            legal_notice = $9, default_language = $10, primary_domain = $11, subdomain = $12,
            https_enabled = $13, currency = $14, tax_mode = $15, tax_rounding = $16,
            order_initial_status = $17, cod_enabled = $18, cod_fee_amount = $19, cod_fee_currency = $20,
            bank_name = $21, bank_branch = $22, bank_account_type = $23, bank_account_number = $24,
            bank_account_name = $25, theme = $26, brand_color = $27, logo_url = $28, favicon_url = $29,
            updated_at = now()
        WHERE store_id = $30
        "#,
    )
    .bind(&settings.store_name)
    .bind(&settings.legal_name)
    .bind(&settings.contact_email)
    .bind(&settings.contact_phone)
    .bind(&settings.address_prefecture)
    .bind(&settings.address_city)
    .bind(&settings.address_line1)
    .bind(&settings.address_line2)
    .bind(&settings.legal_notice)
    .bind(&settings.default_language)
    .bind(&settings.primary_domain)
    .bind(&settings.subdomain)
    .bind(settings.https_enabled)
    .bind(&settings.currency)
    .bind(&settings.tax_mode)
    .bind(&settings.tax_rounding)
    .bind(&settings.order_initial_status)
    .bind(settings.cod_enabled)
    .bind(cod_fee_amount)
    .bind(cod_fee_currency)
    .bind(&settings.bank_name)
    .bind(&settings.bank_branch)
    .bind(&settings.bank_account_type)
    .bind(&settings.bank_account_number)
    .bind(&settings.bank_account_name)
    .bind(&settings.theme)
    .bind(&settings.brand_color)
    .bind(&settings.logo_url)
    .bind(&settings.favicon_url)
    .bind(store_uuid.as_uuid())
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            StoreSettingsAuditAction::Update.into(),
            Some("store_settings"),
            Some(store_id.clone()),
            to_json_opt(before),
            to_json_opt(Some(settings.clone())),
            actor.clone(),
        ),
    )
    .await?;

    Ok(settings)
}

pub async fn initialize_store_settings(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    settings: pb::StoreSettings,
    mall: pb::MallSettings,
    actor: Option<pb::ActorContext>,
) -> Result<(pb::StoreSettings, pb::MallSettings), (StatusCode, Json<ConnectError>)> {
    validate_store_settings(&settings)?;
    validate_mall_settings(&mall)?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let (cod_fee_amount, cod_fee_currency) = money_to_parts(settings.cod_fee.clone())?;
    sqlx::query(
        r#"
        INSERT INTO store_settings (
            store_id, tenant_id, store_name, legal_name, contact_email, contact_phone,
            address_prefecture, address_city, address_line1, address_line2, legal_notice,
            default_language, primary_domain, subdomain, https_enabled, currency,
            tax_mode, tax_rounding, order_initial_status, cod_enabled,
            cod_fee_amount, cod_fee_currency, bank_name, bank_branch, bank_account_type,
            bank_account_number, bank_account_name, theme, brand_color, logo_url, favicon_url
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,
            $11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
            $21,$22,$23,$24,$25,$26,$27,$28,$29,$30,
            $31
        )
        ON CONFLICT (tenant_id) DO NOTHING
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .bind(&settings.store_name)
    .bind(&settings.legal_name)
    .bind(&settings.contact_email)
    .bind(&settings.contact_phone)
    .bind(&settings.address_prefecture)
    .bind(&settings.address_city)
    .bind(&settings.address_line1)
    .bind(&settings.address_line2)
    .bind(&settings.legal_notice)
    .bind(&settings.default_language)
    .bind(&settings.primary_domain)
    .bind(&settings.subdomain)
    .bind(settings.https_enabled)
    .bind(&settings.currency)
    .bind(&settings.tax_mode)
    .bind(&settings.tax_rounding)
    .bind(&settings.order_initial_status)
    .bind(settings.cod_enabled)
    .bind(cod_fee_amount)
    .bind(cod_fee_currency)
    .bind(&settings.bank_name)
    .bind(&settings.bank_branch)
    .bind(&settings.bank_account_type)
    .bind(&settings.bank_account_number)
    .bind(&settings.bank_account_name)
    .bind(&settings.theme)
    .bind(&settings.brand_color)
    .bind(&settings.logo_url)
    .bind(&settings.favicon_url)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            StoreSettingsAuditAction::Initialize.into(),
            Some("store_settings"),
            Some(store_id.clone()),
            None,
            to_json_opt(Some(settings.clone())),
            actor.clone(),
        ),
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO mall_settings (store_id, tenant_id, enabled, commission_rate, vendor_approval_required)
        VALUES ($1,$2,$3,$4,$5)
        ON CONFLICT (tenant_id)
        DO UPDATE SET enabled = EXCLUDED.enabled,
                      commission_rate = EXCLUDED.commission_rate,
                      vendor_approval_required = EXCLUDED.vendor_approval_required
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .bind(mall.enabled)
    .bind(mall.commission_rate)
    .bind(mall.vendor_approval_required)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            MallSettingsAuditAction::Initialize.into(),
            Some("mall_settings"),
            Some(store_id.clone()),
            None,
            to_json_opt(Some(mall.clone())),
            actor.clone(),
        ),
    )
    .await?;

    Ok((settings, mall))
}

pub async fn get_mall_settings(
    state: &AppState,
    store_id: String,
) -> Result<pb::MallSettings, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let row = sqlx::query(
        r#"
        SELECT enabled, commission_rate, vendor_approval_required
        FROM mall_settings
        WHERE store_id = $1
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::MallSettings {
        enabled: row.get("enabled"),
        commission_rate: row.get::<f64, _>("commission_rate"),
        vendor_approval_required: row.get("vendor_approval_required"),
    })
}

pub async fn list_store_locations(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::StoreLocation>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id, code, name, status
        FROM store_locations
        WHERE store_id = $1
        ORDER BY code ASC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::StoreLocation {
            id: row.get("id"),
            code: row.get("code"),
            name: row.get("name"),
            status: row.get("status"),
        })
        .collect())
}

pub async fn upsert_store_location(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    location: pb::StoreLocation,
    actor: Option<pb::ActorContext>,
) -> Result<pb::StoreLocation, (StatusCode, Json<ConnectError>)> {
    validate_store_location(&location)?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let location_id = if location.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&location.id, "location_id")?
    };

    if location.id.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO store_locations (id, tenant_id, store_id, code, name, status)
            VALUES ($1,$2,$3,$4,$5,$6)
        "#,
    )
    .bind(location_id)
    .bind(tenant_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .bind(&location.code)
    .bind(&location.name)
    .bind(&location.status)
    .execute(&state.db)
        .await
        .map_err(db::error)?;
    } else {
        sqlx::query(
            r#"
            UPDATE store_locations
            SET code = $1, name = $2, status = $3, updated_at = now()
            WHERE id = $4 AND store_id = $5
        "#,
    )
    .bind(&location.code)
    .bind(&location.name)
    .bind(&location.status)
    .bind(location_id)
    .bind(store_uuid.as_uuid())
    .execute(&state.db)
    .await
    .map_err(db::error)?;
    }

    let updated = pb::StoreLocation {
        id: location_id.to_string(),
        code: location.code,
        name: location.name,
        status: location.status,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            StoreLocationAuditAction::Upsert.into(),
            Some("store_location"),
            Some(updated.id.clone()),
            None,
            to_json_opt(Some(updated.clone())),
            actor.clone(),
        ),
    )
    .await?;

    Ok(updated)
}

pub async fn delete_store_location(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    location_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    let res = sqlx::query("DELETE FROM store_locations WHERE id = $1 AND store_id = $2")
        .bind(parse_uuid(&location_id, "location_id")?)
        .bind(store_uuid.as_uuid())
        .execute(&state.db)
        .await
        .map_err(db::error)?;
    let deleted = res.rows_affected() > 0;
    if deleted {
        let _ = audit::record(
            state,
            audit_input(
                tenant_id.clone(),
                StoreLocationAuditAction::Delete.into(),
                Some("store_location"),
                Some(location_id),
                None,
                None,
                actor.clone(),
            ),
        )
        .await?;
    }
    Ok(deleted)
}

pub async fn update_mall_settings(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    mall: pb::MallSettings,
    actor: Option<pb::ActorContext>,
) -> Result<pb::MallSettings, (StatusCode, Json<ConnectError>)> {
    validate_mall_settings(&mall)?;
    let before = get_mall_settings(state, store_id.clone()).await.ok();
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    sqlx::query(
        r#"
        UPDATE mall_settings
        SET enabled = $1, commission_rate = $2, vendor_approval_required = $3
        WHERE store_id = $4
        "#,
    )
    .bind(mall.enabled)
    .bind(mall.commission_rate)
    .bind(mall.vendor_approval_required)
    .bind(store_uuid.as_uuid())
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            MallSettingsAuditAction::Update.into(),
            Some("mall_settings"),
            Some(store_id.clone()),
            to_json_opt(before),
            to_json_opt(Some(mall.clone())),
            actor.clone(),
        ),
    )
    .await?;

    Ok(mall)
}

pub async fn list_shipping_zones(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::ShippingZone>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let zones = sqlx::query(
        r#"
        SELECT id::text as id, name, domestic_only
        FROM shipping_zones
        WHERE store_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    let mut result = Vec::new();
    for zone in zones {
        let zone_id: String = zone.get("id");
        let prefs = sqlx::query(
            r#"
            SELECT prefecture_code, prefecture_name
            FROM shipping_zone_prefectures
            WHERE zone_id = $1
            ORDER BY prefecture_code
            "#,
        )
        .bind(parse_uuid(&zone_id, "zone_id")?)
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?;
        let prefectures = prefs
            .into_iter()
            .map(|row| pb::Prefecture {
                code: row.get("prefecture_code"),
                name: row.get("prefecture_name"),
            })
            .collect();

        result.push(pb::ShippingZone {
            id: zone_id,
            name: zone.get("name"),
            domestic_only: zone.get("domestic_only"),
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

    let mut tx = state.db.begin().await.map_err(db::error)?;
    if zone.id.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO shipping_zones (id, store_id, tenant_id, name, domestic_only)
            VALUES ($1,$2,$3,$4,$5)
        "#,
    )
    .bind(zone_id)
    .bind(store_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .bind(&zone.name)
    .bind(zone.domestic_only)
    .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    } else {
        sqlx::query(
            r#"
            UPDATE shipping_zones
            SET name = $1, domestic_only = $2, updated_at = now()
            WHERE id = $3 AND store_id = $4
        "#,
    )
    .bind(&zone.name)
    .bind(zone.domestic_only)
    .bind(zone_id)
    .bind(store_uuid.as_uuid())
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;
        sqlx::query("DELETE FROM shipping_zone_prefectures WHERE zone_id = $1")
            .bind(zone_id)
            .execute(&mut *tx)
            .await
            .map_err(db::error)?;
    }

    for pref in &zone.prefectures {
        sqlx::query(
            r#"
            INSERT INTO shipping_zone_prefectures (zone_id, prefecture_code, prefecture_name)
            VALUES ($1,$2,$3)
            "#,
        )
        .bind(zone_id)
        .bind(&pref.code)
        .bind(&pref.name)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    }

    tx.commit().await.map_err(db::error)?;

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
    let mut tx = state.db.begin().await.map_err(db::error)?;
    sqlx::query("DELETE FROM shipping_zone_prefectures WHERE zone_id = $1")
        .bind(zone_uuid)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    let res = sqlx::query("DELETE FROM shipping_zones WHERE id = $1 AND store_id = $2")
        .bind(zone_uuid)
        .bind(store_uuid.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    tx.commit().await.map_err(db::error)?;
    let deleted = res.rows_affected() > 0;
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
    let rows = sqlx::query(
        r#"
        SELECT r.id::text as id, r.zone_id::text as zone_id, r.name,
               r.min_subtotal_amount, r.max_subtotal_amount,
               r.fee_amount, r.fee_currency
        FROM shipping_rates r
        JOIN shipping_zones z ON z.id = r.zone_id
        WHERE z.store_id = $1 AND r.zone_id = $2
        ORDER BY r.created_at ASC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(parse_uuid(&zone_id, "zone_id")?)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::ShippingRate {
            id: row.get("id"),
            zone_id: row.get("zone_id"),
            name: row.get("name"),
            min_subtotal: row
                .get::<Option<i64>, _>("min_subtotal_amount")
                .zip(row.get::<Option<String>, _>("fee_currency"))
                .map(|(amount, currency)| money_from_parts(amount, currency)),
            max_subtotal: row
                .get::<Option<i64>, _>("max_subtotal_amount")
                .zip(row.get::<Option<String>, _>("fee_currency"))
                .map(|(amount, currency)| money_from_parts(amount, currency)),
            fee: Some(money_from_parts(
                row.get("fee_amount"),
                row.get("fee_currency"),
            )),
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
    if rate.id.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO shipping_rates (
                id, zone_id, name, min_subtotal_amount, max_subtotal_amount,
                fee_amount, fee_currency
            ) VALUES ($1,$2,$3,$4,$5,$6,$7)
            "#,
        )
        .bind(rate_id)
        .bind(parse_uuid(&rate.zone_id, "zone_id")?)
        .bind(&rate.name)
        .bind(min)
        .bind(max)
        .bind(fee_amount)
        .bind(&fee_currency)
        .execute(&state.db)
        .await
        .map_err(db::error)?;
    } else {
        sqlx::query(
            r#"
            UPDATE shipping_rates
            SET name = $1, min_subtotal_amount = $2, max_subtotal_amount = $3,
                fee_amount = $4, fee_currency = $5, updated_at = now()
            WHERE id = $6
            "#,
        )
        .bind(&rate.name)
        .bind(min)
        .bind(max)
        .bind(fee_amount)
        .bind(&fee_currency)
        .bind(rate_id)
        .execute(&state.db)
        .await
        .map_err(db::error)?;
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
    let res = sqlx::query(
        r#"
        DELETE FROM shipping_rates r
        USING shipping_zones z
        WHERE r.zone_id = z.id AND z.store_id = $1 AND r.id = $2
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(parse_uuid(&rate_id, "rate_id")?)
    .execute(&state.db)
    .await
    .map_err(db::error)?;
    let deleted = res.rows_affected() > 0;
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

pub async fn list_tax_rules(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::TaxRule>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id, name, rate, applies_to
        FROM tax_rules
        WHERE store_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::TaxRule {
            id: row.get("id"),
            name: row.get("name"),
            rate: row.get::<f64, _>("rate"),
            applies_to: row.get("applies_to"),
        })
        .collect())
}

pub async fn upsert_tax_rule(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    rule: pb::TaxRule,
    actor: Option<pb::ActorContext>,
) -> Result<pb::TaxRule, (StatusCode, Json<ConnectError>)> {
    validate_tax_rule(&rule)?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let rule_id = if rule.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&rule.id, "rule_id")?
    };
    if rule.id.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO tax_rules (id, store_id, tenant_id, name, rate, applies_to)
            VALUES ($1,$2,$3,$4,$5,$6)
        "#,
    )
    .bind(rule_id)
    .bind(store_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .bind(&rule.name)
    .bind(rule.rate)
    .bind(&rule.applies_to)
    .execute(&state.db)
        .await
        .map_err(db::error)?;
    } else {
        sqlx::query(
            r#"
            UPDATE tax_rules
            SET name = $1, rate = $2, applies_to = $3, updated_at = now()
            WHERE id = $4 AND store_id = $5
        "#,
    )
    .bind(&rule.name)
    .bind(rule.rate)
    .bind(&rule.applies_to)
    .bind(rule_id)
    .bind(store_uuid.as_uuid())
    .execute(&state.db)
    .await
    .map_err(db::error)?;
    }

    let updated = pb::TaxRule {
        id: rule_id.to_string(),
        name: rule.name,
        rate: rule.rate,
        applies_to: rule.applies_to,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            TaxRuleAuditAction::Upsert.into(),
            Some("tax_rule"),
            Some(updated.id.clone()),
            None,
            to_json_opt(Some(updated.clone())),
            actor.clone(),
        ),
    )
    .await?;

    Ok(updated)
}

pub async fn delete_tax_rule(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    rule_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    let res = sqlx::query("DELETE FROM tax_rules WHERE id = $1 AND store_id = $2")
        .bind(parse_uuid(&rule_id, "rule_id")?)
        .bind(store_uuid.as_uuid())
        .execute(&state.db)
        .await
        .map_err(db::error)?;
    let deleted = res.rows_affected() > 0;
    if deleted {
        let _ = audit::record(
            state,
            audit_input(
                tenant_id.clone(),
                TaxRuleAuditAction::Delete.into(),
                Some("tax_rule"),
                Some(rule_id),
                None,
                None,
                actor.clone(),
            ),
        )
        .await?;
    }
    Ok(deleted)
}

fn audit_input(
    tenant_id: String,
    action: AuditAction,
    target_type: Option<&str>,
    target_id: Option<String>,
    before_json: Option<serde_json::Value>,
    after_json: Option<serde_json::Value>,
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

fn to_json_opt<T: serde::Serialize>(value: Option<T>) -> Option<serde_json::Value> {
    value.and_then(|v| serde_json::to_value(v).ok())
}

pub async fn resolve_store_context(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> Result<(String, String), (StatusCode, Json<ConnectError>)> {
    if let Some(ctx) = request_context::current() {
        if let Some(auth_store) = ctx.store_id.as_deref() {
            if let Some(store_id) =
                store.as_ref().and_then(|s| if s.store_id.is_empty() { None } else { Some(s.store_id.as_str()) })
            {
                if store_id != auth_store {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(ConnectError {
                            code: "permission_denied",
                            message: "store_id does not match token".to_string(),
                        }),
                    ));
                }
            }
        }
        if let Some(auth_tenant) = ctx.tenant_id.as_deref() {
            if let Some(tenant_id) =
                tenant.as_ref().and_then(|t| if t.tenant_id.is_empty() { None } else { Some(t.tenant_id.as_str()) })
            {
                if tenant_id != auth_tenant {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(ConnectError {
                            code: "permission_denied",
                            message: "tenant_id does not match token".to_string(),
                        }),
                    ));
                }
            }
        }
    }

    if let Some(store_id) = store.as_ref().and_then(|s| if s.store_id.is_empty() { None } else { Some(s.store_id.as_str()) }) {
        let store_uuid = StoreId::parse(store_id)?;
        let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
            .bind(store_uuid.as_uuid())
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: "invalid_argument",
                    message: "store_id not found".to_string(),
                }),
            ));
        };
        let tenant_id: String = row.get("tenant_id");
        return Ok((store_id.to_string(), tenant_id));
    }
    if let Some(store_code) = store.as_ref().and_then(|s| if s.store_code.is_empty() { None } else { Some(s.store_code.as_str()) }) {
        let row = sqlx::query("SELECT id::text as id, tenant_id::text as tenant_id FROM stores WHERE code = $1")
            .bind(store_code)
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: "invalid_argument",
                    message: "store_code not found".to_string(),
                }),
            ));
        };
        let store_id: String = row.get("id");
        let tenant_id: String = row.get("tenant_id");
        if let Some(ctx) = request_context::current() {
            if let Some(auth_store) = ctx.store_id {
                if auth_store != store_id {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(ConnectError {
                            code: "permission_denied",
                            message: "store_code does not match token".to_string(),
                        }),
                    ));
                }
            }
        }
        return Ok((store_id, tenant_id));
    }
    if let Some(tenant_id) = tenant.and_then(|t| if t.tenant_id.is_empty() { None } else { Some(t.tenant_id) }) {
        let tenant_uuid = TenantId::parse(&tenant_id)?;
        let row = sqlx::query("SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1")
            .bind(tenant_uuid.as_uuid())
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: "invalid_argument",
                    message: "tenant_id not found".to_string(),
                }),
            ));
        };
        let store_id: String = row.get("id");
        return Ok((store_id, tenant_id));
    }
    if let Some(ctx) = request_context::current() {
        if let Some(store_id) = ctx.store_id {
            let store_uuid = StoreId::parse(&store_id)?;
            let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
                .bind(store_uuid.as_uuid())
                .fetch_optional(&state.db)
                .await
                .map_err(db::error)?;
            if let Some(row) = row {
                let tenant_id: String = row.get("tenant_id");
                return Ok((store_id, tenant_id));
            }
        }
        if let Some(tenant_id) = ctx.tenant_id {
            let tenant_uuid = TenantId::parse(&tenant_id)?;
            let row = sqlx::query("SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1")
                .bind(tenant_uuid.as_uuid())
                .fetch_optional(&state.db)
                .await
                .map_err(db::error)?;
            if let Some(row) = row {
                let store_id: String = row.get("id");
                return Ok((store_id, tenant_id));
            }
        }
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ConnectError {
            code: "invalid_argument",
            message: "store.store_id or tenant.tenant_id is required".to_string(),
        }),
    ))
}

pub fn validate_store_settings(
    settings: &pb::StoreSettings,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if settings.store_name.is_empty()
        || settings.legal_name.is_empty()
        || settings.contact_email.is_empty()
        || settings.contact_phone.is_empty()
        || settings.address_prefecture.is_empty()
        || settings.address_city.is_empty()
        || settings.address_line1.is_empty()
        || settings.legal_notice.is_empty()
        || settings.default_language.is_empty()
        || settings.currency.is_empty()
        || settings.tax_mode.is_empty()
        || settings.tax_rounding.is_empty()
        || settings.order_initial_status.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "store settings required fields are missing".to_string(),
            }),
        ));
    }
    Ok(())
}

fn validate_store_location(
    location: &pb::StoreLocation,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if location.code.is_empty() || location.name.is_empty() || location.status.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "location code/name/status are required".to_string(),
            }),
        ));
    }
    Ok(())
}


pub fn validate_mall_settings(
    mall: &pb::MallSettings,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if !(0.0..=1.0).contains(&mall.commission_rate) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "commission_rate must be between 0 and 1".to_string(),
            }),
        ));
    }
    Ok(())
}

pub fn validate_shipping_rate(
    rate: &pb::ShippingRate,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if let (Some(min), Some(max)) = (&rate.min_subtotal, &rate.max_subtotal) {
        if min.amount > max.amount {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: "invalid_argument",
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
                    code: "invalid_argument",
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
                    code: "invalid_argument",
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

pub fn validate_tax_rule(rule: &pb::TaxRule) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if !(0.0..=1.0).contains(&rule.rate) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "tax rate must be between 0 and 1".to_string(),
            }),
        ));
    }
    Ok(())
}
