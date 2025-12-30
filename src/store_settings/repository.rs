use axum::{Json, http::StatusCode};
use sqlx::{Postgres, Row, Transaction};

use crate::{rpc::json::ConnectError, infrastructure::db};

pub struct PgStoreSettingsRepository<'a> {
    db: &'a sqlx::PgPool,
}

#[derive(Debug, Clone)]
pub struct StoreSettingsRecord {
    pub store_name: String,
    pub legal_name: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub address_prefecture: String,
    pub address_city: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub legal_notice: String,
    pub default_language: String,
    pub primary_domain: Option<String>,
    pub subdomain: Option<String>,
    pub https_enabled: bool,
    pub currency: String,
    pub tax_mode: String,
    pub tax_rounding: String,
    pub order_initial_status: String,
    pub cod_enabled: bool,
    pub cod_fee_amount: Option<i64>,
    pub cod_fee_currency: Option<String>,
    pub bank_name: String,
    pub bank_branch: String,
    pub bank_account_type: String,
    pub bank_account_number: String,
    pub bank_account_name: String,
    pub theme: String,
    pub brand_color: String,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub time_zone: String,
}

#[derive(Debug, Clone)]
pub struct MallSettingsRecord {
    pub enabled: bool,
    pub commission_rate: f64,
    pub vendor_approval_required: bool,
}

#[derive(Debug, Clone)]
pub struct StoreLocationRecord {
    pub id: String,
    pub code: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ShippingZoneRecord {
    pub id: String,
    pub name: String,
    pub domestic_only: bool,
}

#[derive(Debug, Clone)]
pub struct PrefectureRecord {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ShippingRateRecord {
    pub id: String,
    pub zone_id: String,
    pub name: String,
    pub min_subtotal_amount: Option<i64>,
    pub max_subtotal_amount: Option<i64>,
    pub fee_amount: i64,
    pub fee_currency: String,
}

#[derive(Debug, Clone)]
pub struct TaxRuleRecord {
    pub id: String,
    pub name: String,
    pub rate: f64,
    pub applies_to: String,
}

#[derive(Debug, Clone)]
pub struct StoreLookupRecord {
    pub store_id: String,
    pub tenant_id: String,
}

pub trait StoreSettingsRepository {
    async fn fetch_store_settings_by_store(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<StoreSettingsRecord>, (StatusCode, Json<ConnectError>)>;

    async fn fetch_store_settings_by_tenant(
        &self,
        tenant_uuid: &uuid::Uuid,
    ) -> Result<Option<StoreSettingsRecord>, (StatusCode, Json<ConnectError>)>;

    async fn fetch_store_name(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<String>, (StatusCode, Json<ConnectError>)>;

    async fn upsert_store_settings(
        &self,
        tenant_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        settings: &crate::pb::pb::StoreSettings,
        cod_fee_amount: i64,
        cod_fee_currency: String,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn insert_store_settings_if_absent(
        &self,
        tenant_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        settings: &crate::pb::pb::StoreSettings,
        cod_fee_amount: i64,
        cod_fee_currency: String,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn upsert_mall_settings(
        &self,
        tenant_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        mall: &crate::pb::pb::MallSettings,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn fetch_mall_settings_by_store(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<MallSettingsRecord>, (StatusCode, Json<ConnectError>)>;

    async fn fetch_mall_settings_by_tenant(
        &self,
        tenant_uuid: &uuid::Uuid,
    ) -> Result<Option<MallSettingsRecord>, (StatusCode, Json<ConnectError>)>;

    async fn list_store_locations(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Vec<StoreLocationRecord>, (StatusCode, Json<ConnectError>)>;

    async fn insert_store_location(
        &self,
        location_id: &uuid::Uuid,
        tenant_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        location: &crate::pb::pb::StoreLocation,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn update_store_location(
        &self,
        location_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        location: &crate::pb::pb::StoreLocation,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn delete_store_location(
        &self,
        location_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
    ) -> Result<u64, (StatusCode, Json<ConnectError>)>;

    async fn list_shipping_zones(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Vec<ShippingZoneRecord>, (StatusCode, Json<ConnectError>)>;

    async fn list_zone_prefectures(
        &self,
        zone_uuid: &uuid::Uuid,
    ) -> Result<Vec<PrefectureRecord>, (StatusCode, Json<ConnectError>)>;

    async fn insert_shipping_zone_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        tenant_uuid: &uuid::Uuid,
        zone: &crate::pb::pb::ShippingZone,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn update_shipping_zone_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        zone: &crate::pb::pb::ShippingZone,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn delete_zone_prefectures_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn insert_zone_prefecture_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
        prefecture: &crate::pb::pb::Prefecture,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn delete_shipping_zone_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
    ) -> Result<u64, (StatusCode, Json<ConnectError>)>;

    async fn list_shipping_rates(
        &self,
        store_uuid: &uuid::Uuid,
        zone_uuid: &uuid::Uuid,
    ) -> Result<Vec<ShippingRateRecord>, (StatusCode, Json<ConnectError>)>;

    async fn insert_shipping_rate(
        &self,
        rate_id: &uuid::Uuid,
        zone_uuid: &uuid::Uuid,
        rate: &crate::pb::pb::ShippingRate,
        fee_amount: i64,
        fee_currency: &str,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn update_shipping_rate(
        &self,
        rate_id: &uuid::Uuid,
        rate: &crate::pb::pb::ShippingRate,
        fee_amount: i64,
        fee_currency: &str,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn delete_shipping_rate(
        &self,
        store_uuid: &uuid::Uuid,
        rate_id: &uuid::Uuid,
    ) -> Result<u64, (StatusCode, Json<ConnectError>)>;

    async fn list_tax_rules(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Vec<TaxRuleRecord>, (StatusCode, Json<ConnectError>)>;

    async fn insert_tax_rule(
        &self,
        rule_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        tenant_uuid: &uuid::Uuid,
        rule: &crate::pb::pb::TaxRule,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn update_tax_rule(
        &self,
        rule_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        rule: &crate::pb::pb::TaxRule,
    ) -> Result<(), (StatusCode, Json<ConnectError>)>;

    async fn delete_tax_rule(
        &self,
        rule_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
    ) -> Result<u64, (StatusCode, Json<ConnectError>)>;

    async fn tenant_id_by_store_id(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<String>, (StatusCode, Json<ConnectError>)>;

    async fn store_by_code(
        &self,
        store_code: &str,
    ) -> Result<Option<StoreLookupRecord>, (StatusCode, Json<ConnectError>)>;

    async fn first_store_by_tenant(
        &self,
        tenant_uuid: &uuid::Uuid,
    ) -> Result<Option<String>, (StatusCode, Json<ConnectError>)>;
}

impl<'a> StoreSettingsRepository for PgStoreSettingsRepository<'a> {
    async fn fetch_store_settings_by_store(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<StoreSettingsRecord>, (StatusCode, Json<ConnectError>)> {
        let row = sqlx::query(
            r#"
            SELECT store_name, legal_name, contact_email, contact_phone,
                   address_prefecture, address_city, address_line1, address_line2,
                   legal_notice, default_language, primary_domain, subdomain, https_enabled,
                   currency, tax_mode, tax_rounding, order_initial_status,
                   cod_enabled, cod_fee_amount, cod_fee_currency,
                   bank_name, bank_branch, bank_account_type, bank_account_number, bank_account_name,
                   theme, brand_color, logo_url, favicon_url, time_zone
            FROM store_settings
            WHERE store_id = $1
            "#,
        )
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(crate::infrastructure::db::error)?;

        Ok(row.map(|row| StoreSettingsRecord {
            store_name: row.get("store_name"),
            legal_name: row.get("legal_name"),
            contact_email: row.get("contact_email"),
            contact_phone: row.get("contact_phone"),
            address_prefecture: row.get("address_prefecture"),
            address_city: row.get("address_city"),
            address_line1: row.get("address_line1"),
            address_line2: row.get("address_line2"),
            legal_notice: row.get("legal_notice"),
            default_language: row.get("default_language"),
            primary_domain: row.get("primary_domain"),
            subdomain: row.get("subdomain"),
            https_enabled: row.get("https_enabled"),
            currency: row.get("currency"),
            tax_mode: row.get("tax_mode"),
            tax_rounding: row.get("tax_rounding"),
            order_initial_status: row.get("order_initial_status"),
            cod_enabled: row.get("cod_enabled"),
            cod_fee_amount: row.get("cod_fee_amount"),
            cod_fee_currency: row.get("cod_fee_currency"),
            bank_name: row.get("bank_name"),
            bank_branch: row.get("bank_branch"),
            bank_account_type: row.get("bank_account_type"),
            bank_account_number: row.get("bank_account_number"),
            bank_account_name: row.get("bank_account_name"),
            theme: row.get("theme"),
            brand_color: row.get("brand_color"),
            logo_url: row.get("logo_url"),
            favicon_url: row.get("favicon_url"),
            time_zone: row.get("time_zone"),
        }))
    }

    async fn fetch_store_settings_by_tenant(
        &self,
        tenant_uuid: &uuid::Uuid,
    ) -> Result<Option<StoreSettingsRecord>, (StatusCode, Json<ConnectError>)> {
        let row = sqlx::query(
            r#"
            SELECT store_name, legal_name, contact_email, contact_phone,
                   address_prefecture, address_city, address_line1, address_line2,
                   legal_notice, default_language, primary_domain, subdomain, https_enabled,
                   currency, tax_mode, tax_rounding, order_initial_status,
                   cod_enabled, cod_fee_amount, cod_fee_currency,
                   bank_name, bank_branch, bank_account_type, bank_account_number, bank_account_name,
                   theme, brand_color, logo_url, favicon_url, time_zone
            FROM store_settings
            WHERE tenant_id = $1
            "#,
        )
        .bind(tenant_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(crate::infrastructure::db::error)?;

        Ok(row.map(|row| StoreSettingsRecord {
            store_name: row.get("store_name"),
            legal_name: row.get("legal_name"),
            contact_email: row.get("contact_email"),
            contact_phone: row.get("contact_phone"),
            address_prefecture: row.get("address_prefecture"),
            address_city: row.get("address_city"),
            address_line1: row.get("address_line1"),
            address_line2: row.get("address_line2"),
            legal_notice: row.get("legal_notice"),
            default_language: row.get("default_language"),
            primary_domain: row.get("primary_domain"),
            subdomain: row.get("subdomain"),
            https_enabled: row.get("https_enabled"),
            currency: row.get("currency"),
            tax_mode: row.get("tax_mode"),
            tax_rounding: row.get("tax_rounding"),
            order_initial_status: row.get("order_initial_status"),
            cod_enabled: row.get("cod_enabled"),
            cod_fee_amount: row.get("cod_fee_amount"),
            cod_fee_currency: row.get("cod_fee_currency"),
            bank_name: row.get("bank_name"),
            bank_branch: row.get("bank_branch"),
            bank_account_type: row.get("bank_account_type"),
            bank_account_number: row.get("bank_account_number"),
            bank_account_name: row.get("bank_account_name"),
            theme: row.get("theme"),
            brand_color: row.get("brand_color"),
            logo_url: row.get("logo_url"),
            favicon_url: row.get("favicon_url"),
            time_zone: row.get("time_zone"),
        }))
    }

    async fn fetch_store_name(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<String>, (StatusCode, Json<ConnectError>)> {
        let row = sqlx::query("SELECT name FROM stores WHERE id = $1")
            .bind(store_uuid)
            .fetch_optional(self.db)
            .await
            .map_err(db::error)?;
        Ok(row.map(|row| row.get("name")))
    }

    async fn upsert_store_settings(
        &self,
        tenant_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        settings: &crate::pb::pb::StoreSettings,
        cod_fee_amount: i64,
        cod_fee_currency: String,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query(
            r#"
            INSERT INTO store_settings (
                tenant_id, store_id, store_name, legal_name, contact_email, contact_phone,
                address_prefecture, address_city, address_line1, address_line2,
                legal_notice, default_language, primary_domain, subdomain, https_enabled,
                currency, tax_mode, tax_rounding, order_initial_status, cod_enabled,
                cod_fee_amount, cod_fee_currency, bank_name, bank_branch, bank_account_type,
                bank_account_number, bank_account_name, theme, brand_color, logo_url, favicon_url, time_zone
            )
            VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,
                $11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
                $21,$22,$23,$24,$25,$26,$27,$28,$29,$30,
                $31,$32
            )
            ON CONFLICT (tenant_id)
            DO UPDATE SET store_id = EXCLUDED.store_id,
                          store_name = EXCLUDED.store_name,
                          legal_name = EXCLUDED.legal_name,
                          contact_email = EXCLUDED.contact_email,
                          contact_phone = EXCLUDED.contact_phone,
                          address_prefecture = EXCLUDED.address_prefecture,
                          address_city = EXCLUDED.address_city,
                          address_line1 = EXCLUDED.address_line1,
                          address_line2 = EXCLUDED.address_line2,
                          legal_notice = EXCLUDED.legal_notice,
                          default_language = EXCLUDED.default_language,
                          primary_domain = EXCLUDED.primary_domain,
                          subdomain = EXCLUDED.subdomain,
                          https_enabled = EXCLUDED.https_enabled,
                          currency = EXCLUDED.currency,
                          tax_mode = EXCLUDED.tax_mode,
                          tax_rounding = EXCLUDED.tax_rounding,
                          order_initial_status = EXCLUDED.order_initial_status,
                          cod_enabled = EXCLUDED.cod_enabled,
                          cod_fee_amount = EXCLUDED.cod_fee_amount,
                          cod_fee_currency = EXCLUDED.cod_fee_currency,
                          bank_name = EXCLUDED.bank_name,
                          bank_branch = EXCLUDED.bank_branch,
                          bank_account_type = EXCLUDED.bank_account_type,
                          bank_account_number = EXCLUDED.bank_account_number,
                          bank_account_name = EXCLUDED.bank_account_name,
                          theme = EXCLUDED.theme,
                          brand_color = EXCLUDED.brand_color,
                          logo_url = EXCLUDED.logo_url,
                          favicon_url = EXCLUDED.favicon_url,
                          time_zone = EXCLUDED.time_zone,
                          updated_at = now()
            "#,
        )
        .bind(tenant_uuid)
        .bind(store_uuid)
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
        .bind(&settings.time_zone)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn insert_store_settings_if_absent(
        &self,
        tenant_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        settings: &crate::pb::pb::StoreSettings,
        cod_fee_amount: i64,
        cod_fee_currency: String,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query(
            r#"
            INSERT INTO store_settings (
                store_id, tenant_id, store_name, legal_name, contact_email, contact_phone,
                address_prefecture, address_city, address_line1, address_line2, legal_notice,
                default_language, primary_domain, subdomain, https_enabled, currency,
                tax_mode, tax_rounding, order_initial_status, cod_enabled,
                cod_fee_amount, cod_fee_currency, bank_name, bank_branch, bank_account_type,
                bank_account_number, bank_account_name, theme, brand_color, logo_url, favicon_url, time_zone
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,
                $11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
                $21,$22,$23,$24,$25,$26,$27,$28,$29,$30,
                $31,$32
            )
            ON CONFLICT (tenant_id) DO NOTHING
            "#,
        )
        .bind(store_uuid)
        .bind(tenant_uuid)
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
        .bind(&settings.time_zone)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn upsert_mall_settings(
        &self,
        tenant_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        mall: &crate::pb::pb::MallSettings,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query(
            r#"
            INSERT INTO mall_settings (tenant_id, store_id, enabled, commission_rate, vendor_approval_required)
            VALUES ($1,$2,$3,$4,$5)
            ON CONFLICT (tenant_id)
            DO UPDATE SET store_id = EXCLUDED.store_id,
                          enabled = EXCLUDED.enabled,
                          commission_rate = EXCLUDED.commission_rate,
                          vendor_approval_required = EXCLUDED.vendor_approval_required,
                          updated_at = now()
            "#,
        )
        .bind(tenant_uuid)
        .bind(store_uuid)
        .bind(mall.enabled)
        .bind(mall.commission_rate)
        .bind(mall.vendor_approval_required)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn fetch_mall_settings_by_store(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<MallSettingsRecord>, (StatusCode, Json<ConnectError>)> {
        let row = sqlx::query(
            r#"
            SELECT enabled, commission_rate, vendor_approval_required
            FROM mall_settings
            WHERE store_id = $1
            "#,
        )
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(db::error)?;
        Ok(row.map(|row| MallSettingsRecord {
            enabled: row.get("enabled"),
            commission_rate: row.get::<f64, _>("commission_rate"),
            vendor_approval_required: row.get("vendor_approval_required"),
        }))
    }

    async fn fetch_mall_settings_by_tenant(
        &self,
        tenant_uuid: &uuid::Uuid,
    ) -> Result<Option<MallSettingsRecord>, (StatusCode, Json<ConnectError>)> {
        let row = sqlx::query(
            r#"
            SELECT enabled, commission_rate, vendor_approval_required
            FROM mall_settings
            WHERE tenant_id = $1
            "#,
        )
        .bind(tenant_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(db::error)?;
        Ok(row.map(|row| MallSettingsRecord {
            enabled: row.get("enabled"),
            commission_rate: row.get::<f64, _>("commission_rate"),
            vendor_approval_required: row.get("vendor_approval_required"),
        }))
    }

    async fn list_store_locations(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Vec<StoreLocationRecord>, (StatusCode, Json<ConnectError>)> {
        let rows = sqlx::query(
            r#"
            SELECT id::text as id, code, name, status
            FROM store_locations
            WHERE store_id = $1
            ORDER BY code ASC
            "#,
        )
        .bind(store_uuid)
        .fetch_all(self.db)
        .await
        .map_err(db::error)?;
        Ok(rows
            .into_iter()
            .map(|row| StoreLocationRecord {
                id: row.get("id"),
                code: row.get("code"),
                name: row.get("name"),
                status: row.get("status"),
            })
            .collect())
    }

    async fn insert_store_location(
        &self,
        location_id: &uuid::Uuid,
        tenant_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        location: &crate::pb::pb::StoreLocation,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query(
            r#"
            INSERT INTO store_locations (id, tenant_id, store_id, code, name, status)
            VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(location_id)
        .bind(tenant_uuid)
        .bind(store_uuid)
        .bind(&location.code)
        .bind(&location.name)
        .bind(&location.status)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn update_store_location(
        &self,
        location_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        location: &crate::pb::pb::StoreLocation,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
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
        .bind(store_uuid)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn delete_store_location(
        &self,
        location_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
    ) -> Result<u64, (StatusCode, Json<ConnectError>)> {
        let res = sqlx::query("DELETE FROM store_locations WHERE id = $1 AND store_id = $2")
            .bind(location_id)
            .bind(store_uuid)
            .execute(self.db)
            .await
            .map_err(db::error)?;
        Ok(res.rows_affected())
    }

    async fn list_shipping_zones(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Vec<ShippingZoneRecord>, (StatusCode, Json<ConnectError>)> {
        let zones = sqlx::query(
            r#"
            SELECT id::text as id, name, domestic_only
            FROM shipping_zones
            WHERE store_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(store_uuid)
        .fetch_all(self.db)
        .await
        .map_err(db::error)?;
        Ok(zones
            .into_iter()
            .map(|row| ShippingZoneRecord {
                id: row.get("id"),
                name: row.get("name"),
                domestic_only: row.get("domestic_only"),
            })
            .collect())
    }

    async fn list_zone_prefectures(
        &self,
        zone_uuid: &uuid::Uuid,
    ) -> Result<Vec<PrefectureRecord>, (StatusCode, Json<ConnectError>)> {
        let prefs = sqlx::query(
            r#"
            SELECT prefecture_code, prefecture_name
            FROM shipping_zone_prefectures
            WHERE zone_id = $1
            ORDER BY prefecture_code
            "#,
        )
        .bind(zone_uuid)
        .fetch_all(self.db)
        .await
        .map_err(db::error)?;
        Ok(prefs
            .into_iter()
            .map(|row| PrefectureRecord {
                code: row.get("prefecture_code"),
                name: row.get("prefecture_name"),
            })
            .collect())
    }

    async fn insert_shipping_zone_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        tenant_uuid: &uuid::Uuid,
        zone: &crate::pb::pb::ShippingZone,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query(
            r#"
            INSERT INTO shipping_zones (id, store_id, tenant_id, name, domestic_only)
            VALUES ($1,$2,$3,$4,$5)
            "#,
        )
        .bind(zone_id)
        .bind(store_uuid)
        .bind(tenant_uuid)
        .bind(&zone.name)
        .bind(zone.domestic_only)
        .execute(exec.as_mut())
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn update_shipping_zone_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        zone: &crate::pb::pb::ShippingZone,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
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
        .bind(store_uuid)
        .execute(exec.as_mut())
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn delete_zone_prefectures_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query("DELETE FROM shipping_zone_prefectures WHERE zone_id = $1")
            .bind(zone_id)
            .execute(exec.as_mut())
            .await
            .map_err(db::error)?;
        Ok(())
    }

    async fn insert_zone_prefecture_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
        prefecture: &crate::pb::pb::Prefecture,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query(
            r#"
            INSERT INTO shipping_zone_prefectures (zone_id, prefecture_code, prefecture_name)
            VALUES ($1,$2,$3)
            "#,
        )
        .bind(zone_id)
        .bind(&prefecture.code)
        .bind(&prefecture.name)
        .execute(exec.as_mut())
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn delete_shipping_zone_tx(
        &self,
        exec: &mut Transaction<'_, Postgres>,
        zone_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
    ) -> Result<u64, (StatusCode, Json<ConnectError>)> {
        let res = sqlx::query("DELETE FROM shipping_zones WHERE id = $1 AND store_id = $2")
            .bind(zone_id)
            .bind(store_uuid)
            .execute(exec.as_mut())
            .await
            .map_err(db::error)?;
        Ok(res.rows_affected())
    }

    async fn list_shipping_rates(
        &self,
        store_uuid: &uuid::Uuid,
        zone_uuid: &uuid::Uuid,
    ) -> Result<Vec<ShippingRateRecord>, (StatusCode, Json<ConnectError>)> {
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
        .bind(store_uuid)
        .bind(zone_uuid)
        .fetch_all(self.db)
        .await
        .map_err(db::error)?;
        Ok(rows
            .into_iter()
            .map(|row| ShippingRateRecord {
                id: row.get("id"),
                zone_id: row.get("zone_id"),
                name: row.get("name"),
                min_subtotal_amount: row.get("min_subtotal_amount"),
                max_subtotal_amount: row.get("max_subtotal_amount"),
                fee_amount: row.get("fee_amount"),
                fee_currency: row.get("fee_currency"),
            })
            .collect())
    }

    async fn insert_shipping_rate(
        &self,
        rate_id: &uuid::Uuid,
        zone_uuid: &uuid::Uuid,
        rate: &crate::pb::pb::ShippingRate,
        fee_amount: i64,
        fee_currency: &str,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query(
            r#"
            INSERT INTO shipping_rates (
                id, zone_id, name, min_subtotal_amount, max_subtotal_amount,
                fee_amount, fee_currency
            ) VALUES ($1,$2,$3,$4,$5,$6,$7)
            "#,
        )
        .bind(rate_id)
        .bind(zone_uuid)
        .bind(&rate.name)
        .bind(min)
        .bind(max)
        .bind(fee_amount)
        .bind(fee_currency)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn update_shipping_rate(
        &self,
        rate_id: &uuid::Uuid,
        rate: &crate::pb::pb::ShippingRate,
        fee_amount: i64,
        fee_currency: &str,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
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
        .bind(fee_currency)
        .bind(rate_id)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn delete_shipping_rate(
        &self,
        store_uuid: &uuid::Uuid,
        rate_id: &uuid::Uuid,
    ) -> Result<u64, (StatusCode, Json<ConnectError>)> {
        let res = sqlx::query(
            r#"
            DELETE FROM shipping_rates r
            USING shipping_zones z
            WHERE r.zone_id = z.id AND z.store_id = $1 AND r.id = $2
            "#,
        )
        .bind(store_uuid)
        .bind(rate_id)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(res.rows_affected())
    }

    async fn list_tax_rules(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Vec<TaxRuleRecord>, (StatusCode, Json<ConnectError>)> {
        let rows = sqlx::query(
            r#"
            SELECT id::text as id, name, rate, applies_to
            FROM tax_rules
            WHERE store_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(store_uuid)
        .fetch_all(self.db)
        .await
        .map_err(db::error)?;
        Ok(rows
            .into_iter()
            .map(|row| TaxRuleRecord {
                id: row.get("id"),
                name: row.get("name"),
                rate: row.get::<f64, _>("rate"),
                applies_to: row.get("applies_to"),
            })
            .collect())
    }

    async fn insert_tax_rule(
        &self,
        rule_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        tenant_uuid: &uuid::Uuid,
        rule: &crate::pb::pb::TaxRule,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        sqlx::query(
            r#"
            INSERT INTO tax_rules (id, store_id, tenant_id, name, rate, applies_to)
            VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(rule_id)
        .bind(store_uuid)
        .bind(tenant_uuid)
        .bind(&rule.name)
        .bind(rule.rate)
        .bind(&rule.applies_to)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn update_tax_rule(
        &self,
        rule_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        rule: &crate::pb::pb::TaxRule,
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
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
        .bind(store_uuid)
        .execute(self.db)
        .await
        .map_err(db::error)?;
        Ok(())
    }

    async fn delete_tax_rule(
        &self,
        rule_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
    ) -> Result<u64, (StatusCode, Json<ConnectError>)> {
        let res = sqlx::query("DELETE FROM tax_rules WHERE id = $1 AND store_id = $2")
            .bind(rule_id)
            .bind(store_uuid)
            .execute(self.db)
            .await
            .map_err(db::error)?;
        Ok(res.rows_affected())
    }

    async fn tenant_id_by_store_id(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<String>, (StatusCode, Json<ConnectError>)> {
        let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
            .bind(store_uuid)
            .fetch_optional(self.db)
            .await
            .map_err(db::error)?;
        Ok(row.map(|row| row.get("tenant_id")))
    }

    async fn store_by_code(
        &self,
        store_code: &str,
    ) -> Result<Option<StoreLookupRecord>, (StatusCode, Json<ConnectError>)> {
        let row =
            sqlx::query("SELECT id::text as id, tenant_id::text as tenant_id FROM stores WHERE code = $1")
                .bind(store_code)
                .fetch_optional(self.db)
                .await
                .map_err(db::error)?;
        Ok(row.map(|row| StoreLookupRecord {
            store_id: row.get("id"),
            tenant_id: row.get("tenant_id"),
        }))
    }

    async fn first_store_by_tenant(
        &self,
        tenant_uuid: &uuid::Uuid,
    ) -> Result<Option<String>, (StatusCode, Json<ConnectError>)> {
        let row = sqlx::query(
            "SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1",
        )
        .bind(tenant_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(db::error)?;
        Ok(row.map(|row| row.get("id")))
    }
}

impl<'a> PgStoreSettingsRepository<'a> {
    pub fn new(db: &'a sqlx::PgPool) -> Self {
        Self { db }
    }
}
