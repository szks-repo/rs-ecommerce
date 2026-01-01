use axum::{Json, http::StatusCode};

use crate::rpc::request_context;
use crate::{
    AppState,
    infrastructure::{audit, db},
    pb::pb,
    rpc::json::ConnectError,
    shared::{
        audit_action::{MallSettingsAuditAction, StoreSettingsAuditAction},
        audit_helpers::{audit_input, to_json_opt},
        ids::{StoreId, TenantId},
        money::{money_from_parts, money_to_parts},
    },
    store_settings::{
        locations,
        repository::{PgStoreSettingsRepository, StoreSettingsRecord, StoreSettingsRepository},
        shipping, tax,
    },
};

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
        return Ok(store_settings_from_record(row));
    }

    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let fallback_row = repo
        .fetch_store_settings_by_tenant(&tenant_uuid.as_uuid())
        .await?;

    if let Some(row) = fallback_row {
        return Ok(store_settings_from_record(row));
    }

    let store_name = repo
        .fetch_store_name(&store_uuid.as_uuid())
        .await?
        .unwrap_or_else(|| "Store".to_string());

    Ok(default_store_settings(store_name))
}

pub(crate) fn default_store_settings(store_name: String) -> pb::StoreSettings {
    pb::StoreSettings {
        profile: Some(pb::StoreProfile {
            store_name: store_name.clone(),
            legal_name: store_name,
            legal_notice: "".to_string(),
        }),
        contact: Some(pb::StoreContact {
            contact_email: "".to_string(),
            contact_phone: "".to_string(),
        }),
        address: Some(pb::StoreAddress {
            address_prefecture: "".to_string(),
            address_city: "".to_string(),
            address_line1: "".to_string(),
            address_line2: "".to_string(),
        }),
        domain: Some(pb::StoreDomain {
            primary_domain: "".to_string(),
            subdomain: "".to_string(),
            https_enabled: true,
        }),
        locale: Some(pb::StoreLocale {
            default_language: "ja".to_string(),
            currency: "JPY".to_string(),
            time_zone: "Asia/Tokyo".to_string(),
        }),
        tax: Some(pb::StoreTax {
            tax_mode: "exclusive".to_string(),
            tax_rounding: "round".to_string(),
        }),
        order: Some(pb::StoreOrder {
            order_initial_status: "pending_payment".to_string(),
        }),
        payment: Some(pb::StorePayment {
            cod_enabled: true,
            cod_fee: Some(money_from_parts(0, "JPY".to_string())),
            bank_transfer_enabled: true,
            bank_account: Some(pb::BankAccount {
                bank_name: "".to_string(),
                bank_branch: "".to_string(),
                bank_account_type: "".to_string(),
                bank_account_number: "".to_string(),
                bank_account_name: "".to_string(),
            }),
        }),
        branding: Some(pb::StoreBranding {
            theme: "default".to_string(),
            brand_color: "#111827".to_string(),
            logo_url: "".to_string(),
            favicon_url: "".to_string(),
        }),
    }
}

fn store_settings_from_record(row: StoreSettingsRecord) -> pb::StoreSettings {
    pb::StoreSettings {
        profile: Some(pb::StoreProfile {
            store_name: row.store_name,
            legal_name: row.legal_name,
            legal_notice: row.legal_notice,
        }),
        contact: Some(pb::StoreContact {
            contact_email: row.contact_email,
            contact_phone: row.contact_phone,
        }),
        address: Some(pb::StoreAddress {
            address_prefecture: row.address_prefecture,
            address_city: row.address_city,
            address_line1: row.address_line1,
            address_line2: row.address_line2.unwrap_or_default(),
        }),
        domain: Some(pb::StoreDomain {
            primary_domain: row.primary_domain.unwrap_or_default(),
            subdomain: row.subdomain.unwrap_or_default(),
            https_enabled: row.https_enabled,
        }),
        locale: Some(pb::StoreLocale {
            default_language: row.default_language,
            currency: row.currency,
            time_zone: row.time_zone,
        }),
        tax: Some(pb::StoreTax {
            tax_mode: row.tax_mode,
            tax_rounding: row.tax_rounding,
        }),
        order: Some(pb::StoreOrder {
            order_initial_status: row.order_initial_status,
        }),
        payment: Some(pb::StorePayment {
            cod_enabled: row.cod_enabled,
            cod_fee: Some(money_from_parts(
                row.cod_fee_amount.unwrap_or_default(),
                row.cod_fee_currency.unwrap_or_else(|| "JPY".to_string()),
            )),
            bank_transfer_enabled: row.bank_transfer_enabled,
            bank_account: Some(pb::BankAccount {
                bank_name: row.bank_name,
                bank_branch: row.bank_branch,
                bank_account_type: row.bank_account_type,
                bank_account_number: row.bank_account_number,
                bank_account_name: row.bank_account_name,
            }),
        }),
        branding: Some(pb::StoreBranding {
            theme: row.theme,
            brand_color: row.brand_color,
            logo_url: row.logo_url.unwrap_or_default(),
            favicon_url: row.favicon_url.unwrap_or_default(),
        }),
    }
}

pub(crate) fn default_mall_settings() -> pb::MallSettings {
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
    let before = get_store_settings(state, store_id.clone(), tenant_id.clone())
        .await
        .ok();
    let merged_settings = if let Some(existing) = before.clone() {
        merge_store_settings(existing, settings)
    } else {
        settings
    };
    validate_store_settings_for_update(before.as_ref(), &merged_settings)?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let payment = merged_settings.payment.clone().unwrap_or_default();
    let (cod_fee_amount, cod_fee_currency) = money_to_parts(payment.cod_fee.clone())?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(db::error)?;
    repo.upsert_store_settings_tx(
        &mut tx,
        &tenant_uuid.as_uuid(),
        &store_uuid.as_uuid(),
        &merged_settings,
        cod_fee_amount,
        cod_fee_currency,
    )
    .await?;

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            StoreSettingsAuditAction::Update.into(),
            Some("store_settings"),
            Some(store_id.clone()),
            to_json_opt(before),
            to_json_opt(Some(merged_settings.clone())),
            actor.clone(),
        ),
    )
    .await?;
    tx.commit().await.map_err(db::error)?;

    Ok(merged_settings)
}

fn merge_store_settings(existing: pb::StoreSettings, mut incoming: pb::StoreSettings) -> pb::StoreSettings {
    incoming.profile = Some(merge_profile(existing.profile, incoming.profile));
    incoming.contact = Some(merge_contact(existing.contact, incoming.contact));
    incoming.address = Some(merge_address(existing.address, incoming.address));
    incoming.domain = Some(merge_domain(existing.domain, incoming.domain));
    incoming.locale = Some(merge_locale(existing.locale, incoming.locale));
    incoming.tax = Some(merge_tax(existing.tax, incoming.tax));
    incoming.order = Some(merge_order(existing.order, incoming.order));
    incoming.payment = Some(merge_payment(existing.payment, incoming.payment));
    incoming.branding = Some(merge_branding(existing.branding, incoming.branding));
    incoming
}

fn merge_profile(existing: Option<pb::StoreProfile>, incoming: Option<pb::StoreProfile>) -> pb::StoreProfile {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.store_name.is_empty() {
        incoming.store_name = existing.store_name;
    }
    if incoming.legal_name.is_empty() {
        incoming.legal_name = existing.legal_name;
    }
    if incoming.legal_notice.is_empty() {
        incoming.legal_notice = existing.legal_notice;
    }
    incoming
}

fn merge_contact(existing: Option<pb::StoreContact>, incoming: Option<pb::StoreContact>) -> pb::StoreContact {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.contact_email.is_empty() {
        incoming.contact_email = existing.contact_email;
    }
    if incoming.contact_phone.is_empty() {
        incoming.contact_phone = existing.contact_phone;
    }
    incoming
}

fn merge_address(existing: Option<pb::StoreAddress>, incoming: Option<pb::StoreAddress>) -> pb::StoreAddress {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.address_prefecture.is_empty() {
        incoming.address_prefecture = existing.address_prefecture;
    }
    if incoming.address_city.is_empty() {
        incoming.address_city = existing.address_city;
    }
    if incoming.address_line1.is_empty() {
        incoming.address_line1 = existing.address_line1;
    }
    if incoming.address_line2.is_empty() {
        incoming.address_line2 = existing.address_line2;
    }
    incoming
}

fn merge_domain(existing: Option<pb::StoreDomain>, incoming: Option<pb::StoreDomain>) -> pb::StoreDomain {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.primary_domain.is_empty() {
        incoming.primary_domain = existing.primary_domain;
    }
    if incoming.subdomain.is_empty() {
        incoming.subdomain = existing.subdomain;
    }
    incoming
}

fn merge_locale(existing: Option<pb::StoreLocale>, incoming: Option<pb::StoreLocale>) -> pb::StoreLocale {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.default_language.is_empty() {
        incoming.default_language = existing.default_language;
    }
    if incoming.currency.is_empty() {
        incoming.currency = existing.currency;
    }
    if incoming.time_zone.is_empty() {
        incoming.time_zone = existing.time_zone;
    }
    incoming
}

fn merge_tax(existing: Option<pb::StoreTax>, incoming: Option<pb::StoreTax>) -> pb::StoreTax {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.tax_mode.is_empty() {
        incoming.tax_mode = existing.tax_mode;
    }
    if incoming.tax_rounding.is_empty() {
        incoming.tax_rounding = existing.tax_rounding;
    }
    incoming
}

fn merge_order(existing: Option<pb::StoreOrder>, incoming: Option<pb::StoreOrder>) -> pb::StoreOrder {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.order_initial_status.is_empty() {
        incoming.order_initial_status = existing.order_initial_status;
    }
    incoming
}

fn merge_payment(existing: Option<pb::StorePayment>, incoming: Option<pb::StorePayment>) -> pb::StorePayment {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.cod_fee.is_none() {
        incoming.cod_fee = existing.cod_fee;
    }
    if incoming.bank_account.is_none() {
        incoming.bank_account = existing.bank_account;
    }
    incoming
}

fn merge_branding(existing: Option<pb::StoreBranding>, incoming: Option<pb::StoreBranding>) -> pb::StoreBranding {
    let existing = existing.unwrap_or_default();
    let mut incoming = incoming.unwrap_or_default();
    if incoming.theme.is_empty() {
        incoming.theme = existing.theme;
    }
    if incoming.brand_color.is_empty() {
        incoming.brand_color = existing.brand_color;
    }
    if incoming.logo_url.is_empty() {
        incoming.logo_url = existing.logo_url;
    }
    if incoming.favicon_url.is_empty() {
        incoming.favicon_url = existing.favicon_url;
    }
    incoming
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
    let payment = settings.payment.clone().unwrap_or_default();
    let (cod_fee_amount, cod_fee_currency) = money_to_parts(payment.cod_fee.clone())?;
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
            Some(store_id.clone()),
            StoreSettingsAuditAction::Initialize.into(),
            Some("store_settings"),
            Some(store_id.clone()),
            None,
            to_json_opt(Some(settings.clone())),
            actor.clone(),
        ),
    )
    .await?;

    repo.upsert_mall_settings_tx(
        &mut tx,
        &tenant_uuid.as_uuid(),
        &store_uuid.as_uuid(),
        &mall,
    )
    .await?;

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
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
    let before = get_mall_settings(state, store_id.clone(), tenant_id.clone())
        .await
        .ok();
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(db::error)?;
    repo.upsert_mall_settings_tx(
        &mut tx,
        &tenant_uuid.as_uuid(),
        &store_uuid.as_uuid(),
        &mall,
    )
    .await?;

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
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
            if let Some(store_id) = store.as_ref().and_then(|s| {
                if s.store_id.is_empty() {
                    None
                } else {
                    Some(s.store_id.as_str())
                }
            }) {
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
            if let Some(tenant_id) = tenant.as_ref().and_then(|t| {
                if t.tenant_id.is_empty() {
                    None
                } else {
                    Some(t.tenant_id.as_str())
                }
            }) {
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

    if let Some(store_id) = store.as_ref().and_then(|s| {
        if s.store_id.is_empty() {
            None
        } else {
            Some(s.store_id.as_str())
        }
    }) {
        let store_uuid = StoreId::parse(store_id)?;
        let repo = PgStoreSettingsRepository::new(&state.db);
        let tenant_id = repo.tenant_id_by_store_id(&store_uuid.as_uuid()).await?;
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
    if let Some(store_code) = store.as_ref().and_then(|s| {
        if s.store_code.is_empty() {
            None
        } else {
            Some(s.store_code.as_str())
        }
    }) {
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
    if let Some(tenant_id) = tenant.and_then(|t| {
        if t.tenant_id.is_empty() {
            None
        } else {
            Some(t.tenant_id)
        }
    }) {
        let tenant_uuid = TenantId::parse(&tenant_id)?;
        let repo = PgStoreSettingsRepository::new(&state.db);
        let store_id = repo.first_store_by_tenant(&tenant_uuid.as_uuid()).await?;
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
            let tenant_id = repo.tenant_id_by_store_id(&store_uuid.as_uuid()).await?;
            if let Some(tenant_id) = tenant_id {
                return Ok((store_id, tenant_id));
            }
        }
        if let Some(tenant_id) = ctx.tenant_id {
            let tenant_uuid = TenantId::parse(&tenant_id)?;
            let repo = PgStoreSettingsRepository::new(&state.db);
            let store_id = repo.first_store_by_tenant(&tenant_uuid.as_uuid()).await?;
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
    let profile = settings.profile.as_ref();
    let contact = settings.contact.as_ref();
    let address = settings.address.as_ref();
    let locale = settings.locale.as_ref();
    let tax = settings.tax.as_ref();
    let order = settings.order.as_ref();
    if profile.is_none()
        || contact.is_none()
        || address.is_none()
        || locale.is_none()
        || tax.is_none()
        || order.is_none()
        || profile.is_some_and(|p| p.store_name.is_empty() || p.legal_name.is_empty() || p.legal_notice.is_empty())
        || contact.is_some_and(|c| c.contact_email.is_empty() || c.contact_phone.is_empty())
        || address.is_some_and(|a| a.address_prefecture.is_empty() || a.address_city.is_empty() || a.address_line1.is_empty())
        || locale.is_some_and(|l| l.default_language.is_empty() || l.currency.is_empty() || l.time_zone.is_empty())
        || tax.is_some_and(|t| t.tax_mode.is_empty() || t.tax_rounding.is_empty())
        || order.is_some_and(|o| o.order_initial_status.is_empty())
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

pub fn validate_store_settings_for_update(
    before: Option<&pb::StoreSettings>,
    settings: &pb::StoreSettings,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let merged_missing = store_settings_missing_required(settings);
    if !merged_missing {
        return validate_store_settings(settings);
    }
    let before_missing = before.map(store_settings_missing_required).unwrap_or(true);
    if before_missing {
        return Ok(());
    }
    validate_store_settings(settings)
}

fn store_settings_missing_required(settings: &pb::StoreSettings) -> bool {
    let profile = settings.profile.as_ref();
    let contact = settings.contact.as_ref();
    let address = settings.address.as_ref();
    let locale = settings.locale.as_ref();
    let tax = settings.tax.as_ref();
    let order = settings.order.as_ref();
    profile.is_none()
        || contact.is_none()
        || address.is_none()
        || locale.is_none()
        || tax.is_none()
        || order.is_none()
        || profile.is_some_and(|p| p.store_name.is_empty() || p.legal_name.is_empty() || p.legal_notice.is_empty())
        || contact.is_some_and(|c| c.contact_email.is_empty() || c.contact_phone.is_empty())
        || address.is_some_and(|a| a.address_prefecture.is_empty() || a.address_city.is_empty() || a.address_line1.is_empty())
        || locale.is_some_and(|l| l.default_language.is_empty() || l.currency.is_empty() || l.time_zone.is_empty())
        || tax.is_some_and(|t| t.tax_mode.is_empty() || t.tax_rounding.is_empty())
        || order.is_some_and(|o| o.order_initial_status.is_empty())
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
