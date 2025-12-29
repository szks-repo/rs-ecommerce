-- Outbox + processing for multi-store customer sync.

CREATE TABLE IF NOT EXISTS outbox_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid REFERENCES stores(id),
    aggregate_type text NOT NULL,
    aggregate_id text NOT NULL,
    event_type text NOT NULL,
    payload_json jsonb NOT NULL,
    status text NOT NULL DEFAULT 'pending', -- pending | processing | published | failed
    idempotency_key text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);

CREATE UNIQUE INDEX IF NOT EXISTS outbox_idempotency_unique
    ON outbox_events (tenant_id, idempotency_key);
CREATE INDEX IF NOT EXISTS outbox_pending_idx
    ON outbox_events (status, created_at);

CREATE TABLE IF NOT EXISTS processed_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid REFERENCES stores(id),
    event_id uuid NOT NULL REFERENCES outbox_events(id),
    processed_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, event_id, store_id)
);

CREATE TABLE IF NOT EXISTS store_sync_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    customer_sync_enabled bool NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO store_sync_settings (store_id, tenant_id, customer_sync_enabled)
SELECT s.id, s.tenant_id, true
FROM stores s
WHERE NOT EXISTS (
    SELECT 1 FROM store_sync_settings ss WHERE ss.store_id = s.id
);
