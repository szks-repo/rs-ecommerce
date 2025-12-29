-- Customer model refinements for cross-store identity.

-- Canonical customers should not require email/name.
ALTER TABLE customers
    ALTER COLUMN email DROP NOT NULL,
    ALTER COLUMN name DROP NOT NULL;

-- Store-level profile fields.
ALTER TABLE customer_profiles
    ADD COLUMN IF NOT EXISTS name text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS email text,
    ADD COLUMN IF NOT EXISTS phone text,
    ADD COLUMN IF NOT EXISTS notes text;

-- Tenant-scoped identity map.
ALTER TABLE customer_identities
    ADD COLUMN IF NOT EXISTS tenant_id uuid REFERENCES tenants(id);

UPDATE customer_identities ci
SET tenant_id = c.tenant_id
FROM customers c
WHERE ci.customer_id = c.id
  AND ci.tenant_id IS NULL;

-- Ensure tenant-scoped uniqueness for identities.
ALTER TABLE customer_identities
    DROP CONSTRAINT IF EXISTS customer_identities_identity_type_identity_value_key;
CREATE UNIQUE INDEX IF NOT EXISTS customer_identities_tenant_identity_unique
    ON customer_identities (tenant_id, identity_type, identity_value);
CREATE INDEX IF NOT EXISTS customer_identities_tenant_idx
    ON customer_identities (tenant_id);

-- Addresses per customer.
CREATE TABLE IF NOT EXISTS customer_addresses (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id uuid NOT NULL REFERENCES customers(id),
    type text NOT NULL, -- shipping | billing
    name text NOT NULL,
    postal_code text NOT NULL,
    prefecture text NOT NULL,
    city text NOT NULL,
    line1 text NOT NULL,
    line2 text,
    phone text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS customer_profiles_store_idx ON customer_profiles (store_id);
CREATE INDEX IF NOT EXISTS customer_profiles_customer_idx ON customer_profiles (customer_id);
CREATE INDEX IF NOT EXISTS customer_addresses_customer_idx ON customer_addresses (customer_id);
