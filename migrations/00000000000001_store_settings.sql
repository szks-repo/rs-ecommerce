CREATE TABLE IF NOT EXISTS store_settings (
    tenant_id uuid PRIMARY KEY REFERENCES tenants(id),
    store_name text NOT NULL,
    legal_name text NOT NULL,
    contact_email text NOT NULL,
    contact_phone text NOT NULL,
    address_prefecture text NOT NULL,
    address_city text NOT NULL,
    address_line1 text NOT NULL,
    address_line2 text,
    legal_notice text NOT NULL,
    default_language text NOT NULL,
    primary_domain text,
    subdomain text,
    https_enabled bool NOT NULL DEFAULT true,
    currency text NOT NULL,
    tax_mode text NOT NULL,
    tax_rounding text NOT NULL,
    order_initial_status text NOT NULL,
    cod_enabled bool NOT NULL DEFAULT true,
    cod_fee_amount bigint NOT NULL DEFAULT 0,
    cod_fee_currency text NOT NULL DEFAULT 'JPY',
    bank_name text NOT NULL,
    bank_branch text NOT NULL,
    bank_account_type text NOT NULL,
    bank_account_number text NOT NULL,
    bank_account_name text NOT NULL,
    theme text NOT NULL,
    brand_color text NOT NULL,
    logo_url text,
    favicon_url text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS shipping_zones (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    name text NOT NULL,
    domestic_only bool NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS shipping_zone_prefectures (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id uuid NOT NULL REFERENCES shipping_zones(id),
    prefecture_code text NOT NULL,
    prefecture_name text NOT NULL
);

CREATE TABLE IF NOT EXISTS shipping_rates (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id uuid NOT NULL REFERENCES shipping_zones(id),
    name text NOT NULL,
    min_subtotal_amount bigint,
    max_subtotal_amount bigint,
    fee_amount bigint NOT NULL,
    fee_currency text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS tax_rules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    name text NOT NULL,
    rate numeric NOT NULL,
    applies_to text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS mall_settings (
    tenant_id uuid PRIMARY KEY REFERENCES tenants(id),
    enabled bool NOT NULL DEFAULT false,
    commission_rate numeric NOT NULL DEFAULT 0,
    vendor_approval_required bool NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_settings_tenant_idx ON store_settings (tenant_id);
CREATE INDEX IF NOT EXISTS shipping_zones_tenant_idx ON shipping_zones (tenant_id);
CREATE INDEX IF NOT EXISTS tax_rules_tenant_idx ON tax_rules (tenant_id);
CREATE INDEX IF NOT EXISTS mall_settings_tenant_idx ON mall_settings (tenant_id);
