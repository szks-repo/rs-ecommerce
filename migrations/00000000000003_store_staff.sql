-- Introduce stores and store_staff for separating store from store_settings.

CREATE TABLE IF NOT EXISTS stores (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    name text NOT NULL,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_staff (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    email text,
    login_id text,
    phone text,
    password_hash text,
    role text NOT NULL,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS stores_tenant_idx ON stores (tenant_id);
CREATE INDEX IF NOT EXISTS store_staff_store_idx ON store_staff (store_id);

-- Allow multiple auth identifiers; each is unique per store if present.
CREATE UNIQUE INDEX IF NOT EXISTS store_staff_email_unique
    ON store_staff (store_id, email) WHERE email IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS store_staff_login_id_unique
    ON store_staff (store_id, login_id) WHERE login_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS store_staff_phone_unique
    ON store_staff (store_id, phone) WHERE phone IS NOT NULL;

-- Add store_id references to settings tables (keep tenant_id for now).
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS store_id uuid REFERENCES stores(id);
ALTER TABLE mall_settings ADD COLUMN IF NOT EXISTS store_id uuid REFERENCES stores(id);
ALTER TABLE shipping_zones ADD COLUMN IF NOT EXISTS store_id uuid REFERENCES stores(id);
ALTER TABLE tax_rules ADD COLUMN IF NOT EXISTS store_id uuid REFERENCES stores(id);

CREATE INDEX IF NOT EXISTS store_settings_store_idx ON store_settings (store_id);
CREATE INDEX IF NOT EXISTS mall_settings_store_idx ON mall_settings (store_id);
CREATE INDEX IF NOT EXISTS shipping_zones_store_idx ON shipping_zones (store_id);
CREATE INDEX IF NOT EXISTS tax_rules_store_idx ON tax_rules (store_id);

-- Backfill stores per tenant (if not exists), then link settings to store.
INSERT INTO stores (id, tenant_id, name, status)
SELECT gen_random_uuid(), t.id, COALESCE(ss.store_name, t.name), 'active'
FROM tenants t
LEFT JOIN store_settings ss ON ss.tenant_id = t.id
WHERE NOT EXISTS (
    SELECT 1 FROM stores s WHERE s.tenant_id = t.id
);

UPDATE store_settings ss
SET store_id = s.id
FROM stores s
WHERE ss.tenant_id = s.tenant_id
  AND ss.store_id IS NULL;

UPDATE mall_settings ms
SET store_id = s.id
FROM stores s
WHERE ms.tenant_id = s.tenant_id
  AND ms.store_id IS NULL;

UPDATE shipping_zones sz
SET store_id = s.id
FROM stores s
WHERE sz.tenant_id = s.tenant_id
  AND sz.store_id IS NULL;

UPDATE tax_rules tr
SET store_id = s.id
FROM stores s
WHERE tr.tenant_id = s.tenant_id
  AND tr.store_id IS NULL;
