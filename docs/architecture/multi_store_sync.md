# Multi-Store Integration (Outbox Pattern)

This document defines async synchronization between stores using the outbox pattern.

## Goals
- Decouple store-to-store data flow with reliable delivery.
- Keep operations idempotent and auditable.
- Fan-out customer data to all stores with sync enabled.

## Scope (Initial)
- **Customer only** (profile + identities).
- Fan-out to all stores where `store_sync_settings.customer_sync_enabled = true`.

## Outbox Pattern Overview
1) Domain operation writes data + an outbox event.
2) Outbox worker reads pending events and applies fan-out.
3) Consumers mark idempotent receipts in `processed_events`.

## Data Model (Outbox)
```sql
CREATE TABLE IF NOT EXISTS outbox_events (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  store_id uuid REFERENCES stores(id),
  aggregate_type text NOT NULL,    -- customer | inventory | product | order
  aggregate_id text NOT NULL,
  event_type text NOT NULL,        -- customer.profile_upsert, customer.identity_upsert
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
```

### Store Sync Settings
```sql
CREATE TABLE IF NOT EXISTS store_sync_settings (
  store_id uuid PRIMARY KEY REFERENCES stores(id),
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  customer_sync_enabled bool NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
```

## Event Payload (Current)
```json
{
  "tenant_id": "uuid",
  "source_store_id": "uuid | null",
  "customer_id": "uuid",
  "profile": { "name": "...", "email": "...", "phone": "...", "status": "...", "notes": "..." }
}
```

## Delivery Strategy
- **Worker**: reads `outbox_events` with `FOR UPDATE SKIP LOCKED` in batches.
- **Ack**: set `published_at`, status=published after successful apply.
- **Retry**: status=failed for manual recovery (can add retry count later).

## Consumers (Initial)
- **Customer Sync Worker**: applies `customer.profile_upsert` across all stores with sync enabled.
- **Identity Upsert**: stored once per tenant (`customer_identities` is tenant-level).

## Idempotency
- Producers write `idempotency_key` into outbox (request_id preferred).
- Consumers maintain a `processed_events` table per store:
```sql
CREATE TABLE IF NOT EXISTS processed_events (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  store_id uuid REFERENCES stores(id),
  event_id uuid NOT NULL,
  processed_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, event_id, store_id)
);
```

## Multi-Store Routing
- For shared DB: worker applies data within the same DB (still async).
- For dedicated DB: route by store to a per-store connection (future).

## Event Types (Initial)
- `customer.profile_upsert`
- `customer.identity_upsert`

## Operational Notes
- Keep event payloads small; use IDs + fetch if needed.
- Add audit logs for each publish/consume with event_id.
- Monitor lag with metrics: pending count, publish latency, failed count.

## Open Questions
- Do we also fan-out customer addresses?
- Add retry/backoff columns for failed outbox events?
