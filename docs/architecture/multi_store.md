# Multi-Store Strategy (Draft)

This document outlines how we support multi-store operations with:
- Cross-store customer identity resolution (same-person matching).
- Flexible database isolation (shared DB for small stores, separate DB for large stores).

## Goals
- Allow customers to be recognized across stores within the same tenant.
- Keep store data isolation flexible by scale.
- Minimize operational complexity for small tenants.

## Customer Identity (Same-Person Matching)

### Core Concept
- A **tenant-level canonical customer** acts as the root identity.
- Store-specific profiles attach to the canonical customer.
- Identity resolution uses verified identifiers (email/phone/external IDs) to link identities.

### Suggested Data Shape (Logical)
#### Canonical + Store Profiles
- `customers` (tenant-level root)
  - `id`, `tenant_id`, `status`, `created_at`, `updated_at`
- `customer_profiles` (store-level view)
  - `id`, `customer_id`, `store_id`, `status`, `preferences_json`, `created_at`, `updated_at`
- `customer_identities` (identity map)
  - `id`, `customer_id`, `identity_type`, `identity_value`, `verified`, `source`, `created_at`

### Matching Flow (High-Level)
1) Normalize identifiers (email/phone).
2) Look up `customer_identities`.
3) If match exists, link to existing `customer_id`.
4) If multiple candidates, flag for manual review or apply deterministic rule.
5) If no match, create a new canonical customer and identity records.

### Safety/Privacy Notes
- Keep PII in a dedicated table or database if required.
- Store-level data can be limited to avoid unnecessary sharing.
- All merges should be auditable (audit log).

### Concrete Schema Draft (DDL Sketch)
```sql
CREATE TABLE IF NOT EXISTS customers (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  status text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS customer_profiles (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  customer_id uuid NOT NULL REFERENCES customers(id),
  store_id uuid NOT NULL REFERENCES stores(id),
  status text NOT NULL,
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
```

## Database Isolation Strategy

### Two Modes
1) **Shared DB (small scale)**
   - All stores for a tenant share the same database instance.
   - `store_id` scoping is mandatory in queries and indexes.
2) **Dedicated DB (large scale)**
   - A store (or tenant) gets a separate DB instance.
   - Routing is based on `store_id`.

### Routing Concept
Maintain a routing registry:
- `store_db_routing`: `store_id -> db_key`
- `db_key -> connection config`
- Application resolves `store_id` for each request and selects the correct pool.

### Data Placement
- **Common DB** (shared across tenant):
  - tenants, identity/auth, canonical customers, billing.
- **Store DB** (per store or shared):
  - products, variants, inventory, orders, carts.

### Concrete Routing Schema (DDL Sketch)
```sql
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
```

### Migration Strategy
- Start with shared DB.
- Promote a store to dedicated DB when scale or regulatory needs demand it.
- Implement data export/import pipeline for store-scoped tables.

### Runtime Pool Strategy (Implementation Notes)
- Cache pools in-memory by `db_key` (LRU or size-limited).
- Resolve `store_id` early in request pipeline.
- For shared DB, route to the default shared pool.
- For dedicated DB, route to store-specific pool.

## API/Runtime Implications
- `store_id` should be required for store-scoped operations.
- Tenant-only operations should be kept minimal.
- Store resolution must be deterministic and validated.

## Open Questions
- How to handle partial identity matches (e.g., same phone but different email)?
- Merge policy (automatic vs manual approval).
- Consistency level between shared vs dedicated DB (eventual vs strong).
