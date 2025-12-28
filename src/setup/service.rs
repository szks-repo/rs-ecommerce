use axum::{Json, http::StatusCode};
use crate::{
    AppState,
    pb::pb,
    infrastructure::db,
    rpc::json::ConnectError,
    shared::money::money_to_parts,
    store_settings,
};

pub async fn initialize_store(
    state: &AppState,
    req: pb::InitializeStoreRequest,
) -> Result<pb::InitializeStoreResponse, (StatusCode, Json<ConnectError>)> {
    let actor = req.actor.clone();
    if req.tenant_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "tenant_name is required".to_string(),
            }),
        ));
    }
    let settings = req.settings.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "settings is required".to_string(),
            }),
        )
    })?;
    let mall = req.mall.unwrap_or(pb::MallSettings {
        enabled: false,
        commission_rate: 0.0,
        vendor_approval_required: true,
    });
    validate_initialize_input(&settings, &mall, &req.default_zone, &req.default_rate, &req.default_tax_rule)?;

    let mut tx = state.db.begin().await.map_err(db::error)?;

    let existing = sqlx::query("SELECT 1 FROM tenants WHERE name = $1 LIMIT 1")
        .bind(&req.tenant_name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::error)?;
    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ConnectError {
                code: "already_exists",
                message: "tenant_name already exists".to_string(),
            }),
        ));
    }

    let tenant_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO tenants (id, name, type, default_currency, status, settings)
        VALUES ($1, $2, $3, $4, $5, '{}'::jsonb)
        "#,
    )
    .bind(tenant_id)
    .bind(&req.tenant_name)
    .bind(if mall.enabled { "mall" } else { "single_brand" })
    .bind(&settings.currency)
    .bind("active")
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    let vendor_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO vendors (id, tenant_id, name, commission_rate, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(vendor_id)
    .bind(tenant_id)
    .bind(&req.tenant_name)
    .bind(mall.commission_rate)
    .bind("active")
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    // Store settings
    let (cod_fee_amount, cod_fee_currency) = money_to_parts(settings.cod_fee.clone())?;
    sqlx::query(
        r#"
        INSERT INTO store_settings (
            tenant_id, store_name, legal_name, contact_email, contact_phone,
            address_prefecture, address_city, address_line1, address_line2, legal_notice,
            default_language, primary_domain, subdomain, https_enabled, currency,
            tax_mode, tax_rounding, order_initial_status, cod_enabled,
            cod_fee_amount, cod_fee_currency, bank_name, bank_branch, bank_account_type,
            bank_account_number, bank_account_name, theme, brand_color, logo_url, favicon_url
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,
            $11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
            $21,$22,$23,$24,$25,$26,$27,$28,$29,$30
        )
        "#,
    )
    .bind(tenant_id)
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
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    sqlx::query(
        r#"
        INSERT INTO mall_settings (tenant_id, enabled, commission_rate, vendor_approval_required)
        VALUES ($1,$2,$3,$4)
        "#,
    )
    .bind(tenant_id)
    .bind(mall.enabled)
    .bind(mall.commission_rate)
    .bind(mall.vendor_approval_required)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    if let Some(zone) = req.default_zone {
        let zone_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO shipping_zones (id, tenant_id, name, domestic_only)
            VALUES ($1,$2,$3,$4)
            "#,
        )
        .bind(zone_id)
        .bind(tenant_id)
        .bind(&zone.name)
        .bind(zone.domestic_only)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;

        for pref in zone.prefectures {
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

        if let Some(rate) = req.default_rate {
            let (fee_amount, fee_currency) = money_to_parts(rate.fee.clone())?;
            let min = rate.min_subtotal.clone().map(|m| m.amount);
            let max = rate.max_subtotal.clone().map(|m| m.amount);
            sqlx::query(
                r#"
                INSERT INTO shipping_rates (
                    id, zone_id, name, min_subtotal_amount, max_subtotal_amount,
                    fee_amount, fee_currency
                ) VALUES ($1,$2,$3,$4,$5,$6,$7)
                "#,
            )
            .bind(uuid::Uuid::new_v4())
            .bind(zone_id)
            .bind(rate.name)
            .bind(min)
            .bind(max)
            .bind(fee_amount)
            .bind(fee_currency)
            .execute(&mut *tx)
            .await
            .map_err(db::error)?;
        }
    }

    if let Some(rule) = req.default_tax_rule {
        sqlx::query(
            r#"
            INSERT INTO tax_rules (id, tenant_id, name, rate, applies_to)
            VALUES ($1,$2,$3,$4,$5)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(tenant_id)
        .bind(rule.name)
        .bind(rule.rate)
        .bind(rule.applies_to)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    }

    tx.commit().await.map_err(db::error)?;

    // Ensure initial store_settings validation/normalization.
    let _ = store_settings::service::update_store_settings(
        state,
        tenant_id.to_string(),
        settings.clone(),
        actor,
    )
    .await?;

    // Ensure search settings exist (safe to call repeatedly).
    let _ = state.search.ensure_settings().await;

    Ok(pb::InitializeStoreResponse {
        tenant_id: tenant_id.to_string(),
        vendor_id: vendor_id.to_string(),
        settings: Some(settings),
        mall: Some(mall),
    })
}

fn validate_initialize_input(
    settings: &pb::StoreSettings,
    mall: &pb::MallSettings,
    zone: &Option<pb::ShippingZone>,
    rate: &Option<pb::ShippingRate>,
    tax_rule: &Option<pb::TaxRule>,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    store_settings::service::validate_store_settings(settings)?;
    store_settings::service::validate_mall_settings(mall)?;

    if let Some(zone) = zone {
        store_settings::service::validate_shipping_zone(zone)?;
    }
    if let Some(rate) = rate {
        store_settings::service::validate_shipping_rate(rate)?;
    }
    if let Some(rule) = tax_rule {
        store_settings::service::validate_tax_rule(rule)?;
    }
    Ok(())
}
