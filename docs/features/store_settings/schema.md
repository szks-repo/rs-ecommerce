# Store Settings DB Schema (Draft)

This schema adds store settings for tenant operations, Japan-focused shipping, and mall settings.

## Core Settings

### store_settings
- tenant_id (uuid, pk, fk -> tenants.id)
- store_name (text)
- legal_name (text)
- contact_email (text)
- contact_phone (text)
- address_prefecture (text)
- address_city (text)
- address_line1 (text)
- address_line2 (text, nullable)
- legal_notice (text) -- 特商法表記
- default_language (text) -- e.g. "ja"
- primary_domain (text, nullable)
- subdomain (text, nullable)
- https_enabled (bool)
- currency (text) -- JPY
- tax_mode (text) -- inclusive | exclusive
- tax_rounding (text) -- floor | round | ceil
- order_initial_status (text) -- pending_payment | pending_shipment
- cod_enabled (bool)
- cod_fee_amount (bigint)
- cod_fee_currency (text)
- bank_name (text)
- bank_branch (text)
- bank_account_type (text)
- bank_account_number (text)
- bank_account_name (text)
- theme (text)
- brand_color (text)
- logo_url (text, nullable)
- favicon_url (text, nullable)
- created_at, updated_at

## Shipping (Japan)

### shipping_zones
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- name (text)
- domestic_only (bool)
- created_at, updated_at

### shipping_zone_prefectures
- id (uuid, pk)
- zone_id (uuid, fk -> shipping_zones.id)
- prefecture_code (text) -- e.g. "JP-13"
- prefecture_name (text) -- "Tokyo"

### shipping_rates
- id (uuid, pk)
- zone_id (uuid, fk -> shipping_zones.id)
- name (text)
- min_subtotal_amount (bigint, nullable)
- max_subtotal_amount (bigint, nullable)
- fee_amount (bigint)
- fee_currency (text)
- created_at, updated_at

## Tax (Optional)

### tax_rules
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- name (text)
- rate (numeric) -- 0.1 for 10%
- applies_to (text) -- all | category | shipping
- created_at, updated_at

## Mall Settings

### mall_settings
- tenant_id (uuid, pk, fk -> tenants.id)
- enabled (bool)
- commission_rate (numeric)
- vendor_approval_required (bool)
- created_at, updated_at

## Indices
- store_settings(tenant_id)
- shipping_zones(tenant_id)
- tax_rules(tenant_id)
- mall_settings(tenant_id)

## Notes
- Use bigint for money amounts (minor units).
- Prefecture codes can follow ISO 3166-2 (JP-01..JP-47).
