DROP TABLE IF EXISTS store_settings;

CREATE TABLE IF NOT EXISTS store_profile_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
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
    order_initial_status text NOT NULL,
    time_zone text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_tax_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    tax_mode text NOT NULL,
    tax_rounding text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_payment_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    cod_enabled bool NOT NULL DEFAULT true,
    cod_fee_amount bigint NOT NULL DEFAULT 0,
    cod_fee_currency text NOT NULL DEFAULT 'JPY',
    bank_transfer_enabled bool NOT NULL DEFAULT true,
    bank_name text NOT NULL,
    bank_branch text NOT NULL,
    bank_account_type text NOT NULL,
    bank_account_number text NOT NULL,
    bank_account_name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_appearance_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    theme text NOT NULL,
    brand_color text NOT NULL,
    logo_url text,
    favicon_url text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_storage_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    storage_provider text NOT NULL DEFAULT '',
    storage_bucket text NOT NULL DEFAULT '',
    storage_base_path text NOT NULL DEFAULT '',
    storage_cdn_base_url text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_profile_settings_tenant_idx ON store_profile_settings (tenant_id);
CREATE INDEX IF NOT EXISTS store_tax_settings_tenant_idx ON store_tax_settings (tenant_id);
CREATE INDEX IF NOT EXISTS store_payment_settings_tenant_idx ON store_payment_settings (tenant_id);
CREATE INDEX IF NOT EXISTS store_appearance_settings_tenant_idx ON store_appearance_settings (tenant_id);
CREATE INDEX IF NOT EXISTS store_storage_settings_tenant_idx ON store_storage_settings (tenant_id);
