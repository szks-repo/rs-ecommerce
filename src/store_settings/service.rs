use axum::{Json, http::StatusCode};

use crate::{
    AppState,
    pb::pb,
    infrastructure::{audit, db},
    rpc::json::ConnectError,
    store_settings::{
        repository::{PgStoreSettingsRepository, StoreSettingsRepository},
        locations,
        shipping,
        tax,
    },
    shared::{
        audit_action::{StoreSettingsAuditAction, MallSettingsAuditAction},
        audit_helpers::{audit_input, to_json_opt},
        ids::{StoreId, TenantId},
        money::{money_from_parts, money_to_parts},
    },
};
use crate::rpc::request_context;

pub struct StoreSettingsService<'a> {
    state: &'a AppState,
}

impl<'a> StoreSettingsService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn get_store_settings(
        &self,
        store_id: String,
        tenant_id: String,
    ) -> Result<pb::StoreSettings, (StatusCode, Json<ConnectError>)> {
        get_store_settings(self.state, store_id, tenant_id).await
    }

    pub async fn update_store_settings(
        &self,
        store_id: String,
        tenant_id: String,
        settings: pb::StoreSettings,
        actor: Option<pb::ActorContext>,
    ) -> Result<pb::StoreSettings, (StatusCode, Json<ConnectError>)> {
        update_store_settings(self.state, store_id, tenant_id, settings, actor).await
    }

    pub async fn initialize_store_settings(
        &self,
        store_id: String,
        tenant_id: String,
        settings: pb::StoreSettings,
        mall: pb::MallSettings,
        actor: Option<pb::ActorContext>,
    ) -> Result<(pb::StoreSettings, pb::MallSettings), (StatusCode, Json<ConnectError>)> {
        initialize_store_settings(self.state, store_id, tenant_id, settings, mall, actor).await
    }

    pub async fn get_mall_settings(
        &self,
        store_id: String,
        tenant_id: String,
    ) -> Result<pb::MallSettings, (StatusCode, Json<ConnectError>)> {
        get_mall_settings(self.state, store_id, tenant_id).await
    }

    pub async fn list_store_locations(
        &self,
        store_id: String,
    ) -> Result<Vec<pb::StoreLocation>, (StatusCode, Json<ConnectError>)> {
        list_store_locations(self.state, store_id).await
    }

    pub async fn upsert_store_location(
        &self,
        store_id: String,
        tenant_id: String,
        location: pb::StoreLocation,
        actor: Option<pb::ActorContext>,
    ) -> Result<pb::StoreLocation, (StatusCode, Json<ConnectError>)> {
        upsert_store_location(self.state, store_id, tenant_id, location, actor).await
    }

    pub async fn delete_store_location(
        &self,
        store_id: String,
        tenant_id: String,
        location_id: String,
        actor: Option<pb::ActorContext>,
    ) -> Result<bool, (StatusCode, Json<ConnectError>)> {
        delete_store_location(self.state, store_id, tenant_id, location_id, actor).await
    }

    pub async fn update_mall_settings(
        &self,
        store_id: String,
        tenant_id: String,
        settings: pb::MallSettings,
        actor: Option<pb::ActorContext>,
    ) -> Result<pb::MallSettings, (StatusCode, Json<ConnectError>)> {
        update_mall_settings(self.state, store_id, tenant_id, settings, actor).await
    }

    pub async fn list_shipping_zones(
        &self,
        store_id: String,
    ) -> Result<Vec<pb::ShippingZone>, (StatusCode, Json<ConnectError>)> {
        list_shipping_zones(self.state, store_id).await
    }

    pub async fn upsert_shipping_zone(
        &self,
        store_id: String,
        tenant_id: String,
        zone: pb::ShippingZone,
        actor: Option<pb::ActorContext>,
    ) -> Result<pb::ShippingZone, (StatusCode, Json<ConnectError>)> {
        upsert_shipping_zone(self.state, store_id, tenant_id, zone, actor).await
    }

    pub async fn delete_shipping_zone(
        &self,
        store_id: String,
        tenant_id: String,
        zone_id: String,
        actor: Option<pb::ActorContext>,
    ) -> Result<bool, (StatusCode, Json<ConnectError>)> {
        delete_shipping_zone(self.state, store_id, tenant_id, zone_id, actor).await
    }

    pub async fn list_shipping_rates(
        &self,
        store_id: String,
        zone_id: String,
    ) -> Result<Vec<pb::ShippingRate>, (StatusCode, Json<ConnectError>)> {
        list_shipping_rates(self.state, store_id, zone_id).await
    }

    pub async fn upsert_shipping_rate(
        &self,
        store_id: String,
        zone_id: String,
        rate: pb::ShippingRate,
        actor: Option<pb::ActorContext>,
    ) -> Result<pb::ShippingRate, (StatusCode, Json<ConnectError>)> {
        upsert_shipping_rate(self.state, store_id, zone_id, rate, actor).await
    }

    pub async fn delete_shipping_rate(
        &self,
        store_id: String,
        zone_id: String,
        rate_id: String,
        actor: Option<pb::ActorContext>,
    ) -> Result<bool, (StatusCode, Json<ConnectError>)> {
        delete_shipping_rate(self.state, store_id, zone_id, rate_id, actor).await
    }

    pub async fn list_tax_rules(
        &self,
        store_id: String,
    ) -> Result<Vec<pb::TaxRule>, (StatusCode, Json<ConnectError>)> {
        list_tax_rules(self.state, store_id).await
    }

    pub async fn upsert_tax_rule(
        &self,
        store_id: String,
        tenant_id: String,
        rule: pb::TaxRule,
        actor: Option<pb::ActorContext>,
    ) -> Result<pb::TaxRule, (StatusCode, Json<ConnectError>)> {
        upsert_tax_rule(self.state, store_id, tenant_id, rule, actor).await
    }

    pub async fn delete_tax_rule(
        &self,
        store_id: String,
        tenant_id: String,
        rule_id: String,
        actor: Option<pb::ActorContext>,
    ) -> Result<bool, (StatusCode, Json<ConnectError>)> {
        delete_tax_rule(self.state, store_id, tenant_id, rule_id, actor).await
    }
}

pub async fn get_store_settings(
    state: &AppState,
    store_id: String,
    tenant_id: String,
) -> Result<pb::StoreSettings, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let row = repo
        .fetch_store_settings_by_store(&store_uuid.as_uuid())
        .await?;

    if let Some(row) = row {
        return Ok(pb::StoreSettings {
            store_name: row.store_name,
            legal_name: row.legal_name,
            contact_email: row.contact_email,
            contact_phone: row.contact_phone,
            address_prefecture: row.address_prefecture,
            address_city: row.address_city,
            address_line1: row.address_line1,
            address_line2: row.address_line2.unwrap_or_default(),
            legal_notice: row.legal_notice,
            default_language: row.default_language,
            primary_domain: row.primary_domain.unwrap_or_default(),
            subdomain: row.subdomain.unwrap_or_default(),
            https_enabled: row.https_enabled,
            currency: row.currency,
            tax_mode: row.tax_mode,
            tax_rounding: row.tax_rounding,
            order_initial_status: row.order_initial_status,
            cod_enabled: row.cod_enabled,
            cod_fee: Some(money_from_parts(
                row.cod_fee_amount.unwrap_or_default(),
                row.cod_fee_currency.unwrap_or_else(|| "JPY".to_string()),
            )),
            bank_name: row.bank_name,
            bank_branch: row.bank_branch,
            bank_account_type: row.bank_account_type,
            bank_account_number: row.bank_account_number,
            bank_account_name: row.bank_account_name,
            theme: row.theme,
            brand_color: row.brand_color,
            logo_url: row.logo_url.unwrap_or_default(),
            favicon_url: row.favicon_url.unwrap_or_default(),
            time_zone: row.time_zone,
        });
    }

    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let fallback_row = repo
        .fetch_store_settings_by_tenant(&tenant_uuid.as_uuid())
        .await?;

    if let Some(row) = fallback_row {
        return Ok(pb::StoreSettings {
            store_name: row.store_name,
            legal_name: row.legal_name,
            contact_email: row.contact_email,
            contact_phone: row.contact_phone,
            address_prefecture: row.address_prefecture,
            address_city: row.address_city,
            address_line1: row.address_line1,
            address_line2: row.address_line2.unwrap_or_default(),
            legal_notice: row.legal_notice,
            default_language: row.default_language,
            primary_domain: row.primary_domain.unwrap_or_default(),
            subdomain: row.subdomain.unwrap_or_default(),
            https_enabled: row.https_enabled,
            currency: row.currency,
            tax_mode: row.tax_mode,
            tax_rounding: row.tax_rounding,
            order_initial_status: row.order_initial_status,
            cod_enabled: row.cod_enabled,
            cod_fee: Some(money_from_parts(
                row.cod_fee_amount.unwrap_or_default(),
                row.cod_fee_currency.unwrap_or_else(|| "JPY".to_string()),
            )),
            bank_name: row.bank_name,
            bank_branch: row.bank_branch,
            bank_account_type: row.bank_account_type,
            bank_account_number: row.bank_account_number,
            bank_account_name: row.bank_account_name,
            theme: row.theme,
            brand_color: row.brand_color,
            logo_url: row.logo_url.unwrap_or_default(),
            favicon_url: row.favicon_url.unwrap_or_default(),
            time_zone: row.time_zone,
        });
    }

    let store_name = repo
        .fetch_store_name(&store_uuid.as_uuid())
        .await?
        .unwrap_or_else(|| "Store".to_string());

    Ok(default_store_settings(store_name))
}

fn default_store_settings(store_name: String) -> pb::StoreSettings {
    pb::StoreSettings {
        store_name: store_name.clone(),
        legal_name: store_name,
        contact_email: "".to_string(),
        contact_phone: "".to_string(),
        address_prefecture: "".to_string(),
        address_city: "".to_string(),
        address_line1: "".to_string(),
        address_line2: "".to_string(),
        legal_notice: "".to_string(),
        default_language: "ja".to_string(),
        primary_domain: "".to_string(),
        subdomain: "".to_string(),
        https_enabled: true,
        currency: "JPY".to_string(),
        tax_mode: "exclusive".to_string(),
        tax_rounding: "round".to_string(),
        order_initial_status: "pending_payment".to_string(),
        cod_enabled: true,
        cod_fee: Some(money_from_parts(0, "JPY".to_string())),
        bank_name: "".to_string(),
        bank_branch: "".to_string(),
        bank_account_type: "".to_string(),
        bank_account_number: "".to_string(),
        bank_account_name: "".to_string(),
        theme: "default".to_string(),
        brand_color: "#111827".to_string(),
        logo_url: "".to_string(),
        favicon_url: "".to_string(),
        time_zone: "Asia/Tokyo".to_string(),
    }
}

fn default_mall_settings() -> pb::MallSettings {
    pb::MallSettings {
        enabled: false,
        commission_rate: 0.0,
        vendor_approval_required: true,
    }
}

pub async fn update_store_settings(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    settings: pb::StoreSettings,
    actor: Option<pb::ActorContext>,
) -> Result<pb::StoreSettings, (StatusCode, Json<ConnectError>)> {
    validate_store_settings(&settings)?;
    let before = get_store_settings(state, store_id.clone(), tenant_id.clone()).await.ok();
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let (cod_fee_amount, cod_fee_currency) = money_to_parts(settings.cod_fee.clone())?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(db::error)?;
    repo.upsert_store_settings_tx(
        &mut tx,
        &tenant_uuid.as_uuid(),
        &store_uuid.as_uuid(),
        &settings,
        cod_fee_amount,
        cod_fee_currency,
    )
    .await?;

    audit::record_tx(
        &mut tx,
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
    tx.commit().await.map_err(db::error)?;

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
    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(db::error)?;
    repo.insert_store_settings_if_absent_tx(
        &mut tx,
        &tenant_uuid.as_uuid(),
        &store_uuid.as_uuid(),
        &settings,
        cod_fee_amount,
        cod_fee_currency,
    )
    .await?;

    audit::record_tx(
        &mut tx,
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

    repo.upsert_mall_settings_tx(&mut tx, &tenant_uuid.as_uuid(), &store_uuid.as_uuid(), &mall)
        .await?;

    audit::record_tx(
        &mut tx,
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
    tx.commit().await.map_err(db::error)?;

    Ok((settings, mall))
}

pub async fn get_mall_settings(
    state: &AppState,
    store_id: String,
    tenant_id: String,
) -> Result<pb::MallSettings, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    if let Some(row) = repo
        .fetch_mall_settings_by_store(&store_uuid.as_uuid())
        .await?
    {
        return Ok(pb::MallSettings {
            enabled: row.enabled,
            commission_rate: row.commission_rate,
            vendor_approval_required: row.vendor_approval_required,
        });
    }

    let tenant_uuid = TenantId::parse(&tenant_id)?;
    if let Some(row) = repo
        .fetch_mall_settings_by_tenant(&tenant_uuid.as_uuid())
        .await?
    {
        return Ok(pb::MallSettings {
            enabled: row.enabled,
            commission_rate: row.commission_rate,
            vendor_approval_required: row.vendor_approval_required,
        });
    }

    Ok(default_mall_settings())
}

pub async fn list_store_locations(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::StoreLocation>, (StatusCode, Json<ConnectError>)> {
    locations::list_store_locations(state, store_id).await
}

pub async fn upsert_store_location(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    location: pb::StoreLocation,
    actor: Option<pb::ActorContext>,
) -> Result<pb::StoreLocation, (StatusCode, Json<ConnectError>)> {
    locations::upsert_store_location(state, store_id, tenant_id, location, actor).await
}

pub async fn delete_store_location(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    location_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    locations::delete_store_location(state, store_id, tenant_id, location_id, actor).await
}

pub async fn update_mall_settings(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    mall: pb::MallSettings,
    actor: Option<pb::ActorContext>,
) -> Result<pb::MallSettings, (StatusCode, Json<ConnectError>)> {
    validate_mall_settings(&mall)?;
    let before = get_mall_settings(state, store_id.clone(), tenant_id.clone()).await.ok();
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(db::error)?;
    repo.upsert_mall_settings_tx(&mut tx, &tenant_uuid.as_uuid(), &store_uuid.as_uuid(), &mall)
        .await?;

    audit::record_tx(
        &mut tx,
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
    tx.commit().await.map_err(db::error)?;

    Ok(mall)
}

pub async fn list_shipping_zones(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::ShippingZone>, (StatusCode, Json<ConnectError>)> {
    shipping::list_shipping_zones(state, store_id).await
}

pub async fn upsert_shipping_zone(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    zone: pb::ShippingZone,
    actor: Option<pb::ActorContext>,
) -> Result<pb::ShippingZone, (StatusCode, Json<ConnectError>)> {
    shipping::upsert_shipping_zone(state, store_id, tenant_id, zone, actor).await
}

pub async fn delete_shipping_zone(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    zone_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    shipping::delete_shipping_zone(state, store_id, tenant_id, zone_id, actor).await
}

pub async fn list_shipping_rates(
    state: &AppState,
    store_id: String,
    zone_id: String,
) -> Result<Vec<pb::ShippingRate>, (StatusCode, Json<ConnectError>)> {
    shipping::list_shipping_rates(state, store_id, zone_id).await
}

pub async fn upsert_shipping_rate(
    state: &AppState,
    _store_id: String,
    tenant_id: String,
    rate: pb::ShippingRate,
    actor: Option<pb::ActorContext>,
) -> Result<pb::ShippingRate, (StatusCode, Json<ConnectError>)> {
    shipping::upsert_shipping_rate(state, _store_id, tenant_id, rate, actor).await
}

pub async fn delete_shipping_rate(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    rate_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    shipping::delete_shipping_rate(state, store_id, tenant_id, rate_id, actor).await
}

pub async fn list_tax_rules(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::TaxRule>, (StatusCode, Json<ConnectError>)> {
    tax::list_tax_rules(state, store_id).await
}

pub async fn upsert_tax_rule(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    rule: pb::TaxRule,
    actor: Option<pb::ActorContext>,
) -> Result<pb::TaxRule, (StatusCode, Json<ConnectError>)> {
    tax::upsert_tax_rule(state, store_id, tenant_id, rule, actor).await
}

pub async fn delete_tax_rule(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    rule_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    tax::delete_tax_rule(state, store_id, tenant_id, rule_id, actor).await
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
                            code: crate::rpc::json::ErrorCode::PermissionDenied,
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
                            code: crate::rpc::json::ErrorCode::PermissionDenied,
                            message: "tenant_id does not match token".to_string(),
                        }),
                    ));
                }
            }
        }
    }

    if let Some(store_id) = store.as_ref().and_then(|s| if s.store_id.is_empty() { None } else { Some(s.store_id.as_str()) }) {
        let store_uuid = StoreId::parse(store_id)?;
        let repo = PgStoreSettingsRepository::new(&state.db);
        let tenant_id = repo
            .tenant_id_by_store_id(&store_uuid.as_uuid())
            .await?;
        let Some(tenant_id) = tenant_id else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_id not found".to_string(),
                }),
            ));
        };
        return Ok((store_id.to_string(), tenant_id));
    }
    if let Some(store_code) = store.as_ref().and_then(|s| if s.store_code.is_empty() { None } else { Some(s.store_code.as_str()) }) {
        let repo = PgStoreSettingsRepository::new(&state.db);
        let row = repo.store_by_code(store_code).await?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_code not found".to_string(),
                }),
            ));
        };
        let store_id = row.store_id;
        let tenant_id = row.tenant_id;
        if let Some(ctx) = request_context::current() {
            if let Some(auth_store) = ctx.store_id {
                if auth_store != store_id {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(ConnectError {
                            code: crate::rpc::json::ErrorCode::PermissionDenied,
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
        let repo = PgStoreSettingsRepository::new(&state.db);
        let store_id = repo
            .first_store_by_tenant(&tenant_uuid.as_uuid())
            .await?;
        let Some(store_id) = store_id else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "tenant_id not found".to_string(),
                }),
            ));
        };
        return Ok((store_id, tenant_id));
    }
    if let Some(ctx) = request_context::current() {
        if let Some(store_id) = ctx.store_id {
            let store_uuid = StoreId::parse(&store_id)?;
            let repo = PgStoreSettingsRepository::new(&state.db);
            let tenant_id = repo
                .tenant_id_by_store_id(&store_uuid.as_uuid())
                .await?;
            if let Some(tenant_id) = tenant_id {
                return Ok((store_id, tenant_id));
            }
        }
        if let Some(tenant_id) = ctx.tenant_id {
            let tenant_uuid = TenantId::parse(&tenant_id)?;
            let repo = PgStoreSettingsRepository::new(&state.db);
            let store_id = repo
                .first_store_by_tenant(&tenant_uuid.as_uuid())
                .await?;
            if let Some(store_id) = store_id {
                return Ok((store_id, tenant_id));
            }
        }
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ConnectError {
            code: crate::rpc::json::ErrorCode::InvalidArgument,
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
        || settings.time_zone.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "store settings required fields are missing".to_string(),
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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "commission_rate must be between 0 and 1".to_string(),
            }),
        ));
    }
    Ok(())
}
