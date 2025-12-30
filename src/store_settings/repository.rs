use axum::{Json, http::StatusCode};
use sqlx::Row;

use crate::rpc::json::ConnectError;

pub struct PgStoreSettingsRepository<'a> {
    db: &'a sqlx::PgPool,
}

pub struct StoreSettingsRow {
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

pub trait StoreSettingsRepository {
    async fn fetch_store_settings_by_store(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<StoreSettingsRow>, (StatusCode, Json<ConnectError>)>;

    async fn fetch_store_settings_by_tenant(
        &self,
        tenant_uuid: &uuid::Uuid,
    ) -> Result<Option<StoreSettingsRow>, (StatusCode, Json<ConnectError>)>;

    async fn fetch_store_name(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<String>, (StatusCode, Json<ConnectError>)>;
}

impl<'a> StoreSettingsRepository for PgStoreSettingsRepository<'a> {
    async fn fetch_store_settings_by_store(
        &self,
        store_uuid: &uuid::Uuid,
    ) -> Result<Option<StoreSettingsRow>, (StatusCode, Json<ConnectError>)> {
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

        Ok(row.map(|row| StoreSettingsRow {
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
    ) -> Result<Option<StoreSettingsRow>, (StatusCode, Json<ConnectError>)> {
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

        Ok(row.map(|row| StoreSettingsRow {
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
            .map_err(crate::infrastructure::db::error)?;
        Ok(row.map(|row| row.get("name")))
    }
}

impl<'a> PgStoreSettingsRepository<'a> {
    pub fn new(db: &'a sqlx::PgPool) -> Self {
        Self { db }
    }
}
