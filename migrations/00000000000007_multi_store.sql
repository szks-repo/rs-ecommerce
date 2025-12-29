-- Multi-store routing + cross-store customer identity.

ALTER TABLE customers
    ADD COLUMN IF NOT EXISTS status text NOT NULL DEFAULT 'active';

CREATE TABLE IF NOT EXISTS customer_profiles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id uuid NOT NULL REFERENCES customers(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    status text NOT NULL DEFAULT 'active',
    preferences jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (customer_id, store_id)
);

CREATE TABLE IF NOT EXISTS customer_identities (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id uuid NOT NULL REFERENCES customers(id),
    identity_type text NOT NULL, -- email | phone | external
    identity_value text NOT NULL, -- normalized
    verified boolean NOT NULL DEFAULT false,
    source text NOT NULL, -- signup | import | merge
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (identity_type, identity_value)
);

CREATE TABLE IF NOT EXISTS store_db_routing (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    db_key text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS db_connections (
    db_key text PRIMARY KEY,
    kind text NOT NULL, -- shared | dedicated
    host text NOT NULL,
    port int NOT NULL,
    database_name text NOT NULL,
    username text NOT NULL,
    password_secret_ref text NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
