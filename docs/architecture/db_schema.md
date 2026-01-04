# Database Schema (Draft)

This schema prioritizes single-brand stores while keeping mall/multi-vendor optional.
Tenancy is enforced with `tenant_id` on all core tables.

## Conventions
- Primary keys: `id` (UUID)
- Timestamps: `created_at`, `updated_at`
- Soft delete where needed: `deleted_at`
- All core tables include `tenant_id`

## Core Tenancy

### tenants
- id (uuid, pk)
- name (text)
- type (text) -- single_brand | mall
- default_currency (text)
- status (text)
- settings (jsonb)
- created_at, updated_at

### vendors
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- name (text)
- commission_rate (numeric)
- status (text)
- created_at, updated_at

Note: In single-brand mode, a tenant can have exactly one vendor representing the store itself.

## Stores & Staff

### stores
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- name (text)
- status (text)
- created_at, updated_at

### store_staff
- id (uuid, pk)
- store_id (uuid, fk -> stores.id)
- email (text, nullable)
- login_id (text, nullable)
- phone (text, nullable)
- password_hash (text, nullable)
- role (text) -- owner | admin | staff
- status (text)
- created_at, updated_at

Note: `store_settings` and related configuration tables are linked by `store_id` (keeping `tenant_id` for now).

### store_locations
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- store_id (uuid, fk -> stores.id)
- code (text, unique per store)
- name (text)
- status (text)
- created_at, updated_at

## Product

### products
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- store_id (uuid, fk -> stores.id)
- vendor_id (uuid, fk -> vendors.id)
- title (text)
- description (text)
- status (text) -- draft | active | archived
- created_at, updated_at

### product_locations
- product_id (uuid, fk -> products.id)
- location_id (uuid, fk -> store_locations.id)

### store_digital_settings
- store_id (uuid, fk -> stores.id)
- default_url_ttl_seconds (int)
- default_max_downloads (int, nullable)
- updated_at

### product_digital_settings
- product_id (uuid, fk -> products.id)
- url_ttl_seconds (int, nullable)
- max_downloads (int, nullable)
- updated_at

### variants
- id (uuid, pk)
- product_id (uuid, fk -> products.id)
- sku (text, unique per product)
- fulfillment_type (text) -- physical | digital
- price_amount (bigint)
- price_currency (text)
- compare_at_amount (bigint, nullable)
- compare_at_currency (text, nullable)
- status (text)
- created_at, updated_at

## Inventory

### inventory_stocks
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- store_id (uuid, fk -> stores.id)
- location_id (uuid, fk -> store_locations.id)
- variant_id (uuid, fk -> variants.id)
- stock (int)
- reserved (int)
- updated_at

### inventory_reservations
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- store_id (uuid, fk -> stores.id)
- cart_id (uuid, fk -> carts.id)
- cart_item_id (uuid, fk -> cart_items.id, nullable)
- variant_id (uuid, fk -> variants.id)
- quantity (int)
- status (text) -- active | expired | consumed | released
- expires_at (timestamptz)
- created_at, updated_at

### inventory_reservation_requests (async queue)
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- store_id (uuid, fk -> stores.id)
- cart_id (uuid, fk -> carts.id)
- cart_item_id (uuid, fk -> cart_items.id, nullable)
- variant_id (uuid, fk -> variants.id)
- quantity (int)
- status (text) -- queued | processing | done | failed
- is_hot (bool)
- idempotency_key (text)
- created_at, updated_at

## Customers (Cross-Store Identity)

### customers (canonical)
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- status (text)
- created_at, updated_at

### customer_profiles (store-level)
- id (uuid, pk)
- customer_id (uuid, fk -> customers.id)
- store_id (uuid, fk -> stores.id)
- name (text)
- email (text, nullable)
- phone (text, nullable)
- status (text)
- notes (text, nullable)
- preferences (jsonb)
- created_at, updated_at

### customer_identities (identity map)
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- customer_id (uuid, fk -> customers.id)
- identity_type (text) -- email | phone | external
- identity_value (text, normalized)
- verified (bool)
- source (text) -- signup | import | merge
- created_at

### customer_addresses
- id (uuid, pk)
- customer_id (uuid, fk -> customers.id)
- type (text) -- shipping | billing
- name (text)
- postal_code (text)
- prefecture (text)
- city (text)
- line1 (text)
- line2 (text, nullable)
- phone (text, nullable)
- created_at, updated_at

### store_sync_settings
- store_id (uuid, pk, fk -> stores.id)
- tenant_id (uuid, fk -> tenants.id)
- customer_sync_enabled (bool)
- created_at, updated_at

### outbox_events
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- store_id (uuid, fk -> stores.id, nullable)
- aggregate_type (text)
- aggregate_id (text)
- event_type (text)
- payload_json (jsonb)
- status (text) -- pending | processing | published | failed
- idempotency_key (text)
- created_at, published_at

### processed_events
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- store_id (uuid, fk -> stores.id, nullable)
- event_id (uuid, fk -> outbox_events.id)
- processed_at (timestamptz)

## Cart / Checkout

### carts
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- customer_id (uuid, fk -> customers.id, nullable)
- status (text) -- active | ordered | abandoned
- created_at, updated_at

### cart_items
- id (uuid, pk)
- cart_id (uuid, fk -> carts.id)
- vendor_id (uuid, fk -> vendors.id)
- variant_id (uuid, fk -> variants.id)
- price (numeric)
- quantity (int)

## Orders

### orders
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- customer_id (uuid, fk -> customers.id)
- status (text)
- total (numeric)
- currency (text)
- payment_method (text) -- bank_transfer | cod
- created_at, updated_at

### order_items
- id (uuid, pk)
- order_id (uuid, fk -> orders.id)
- vendor_id (uuid, fk -> vendors.id)
- variant_id (uuid, fk -> variants.id)
- price_amount (bigint)
- price_currency (text)
- quantity (int)

### digital_deliveries
- id (uuid, pk)
- order_item_id (uuid, fk -> order_items.id)
- provider (text) -- gcs | s3 | other
- object_key (text)
- url_expires_at (timestamptz, nullable)
- max_downloads (int, nullable)
- downloaded_count (int)
- status (text) -- active | expired | revoked
- created_at, updated_at

### order_addresses
- id (uuid, pk)
- order_id (uuid, fk -> orders.id)
- type (text) -- shipping | billing
- name (text)
- postal_code (text)
- prefecture (text)
- city (text)
- line1 (text)
- line2 (text, nullable)
- phone (text, nullable)

## Payments

### payments
- id (uuid, pk)
- order_id (uuid, fk -> orders.id)
- method (text) -- bank_transfer | cod
- status (text) -- pending | confirmed | failed
- amount (numeric)
- created_at, updated_at

## Fulfillment

### shipments
- id (uuid, pk)
- order_id (uuid, fk -> orders.id)
- vendor_id (uuid, fk -> vendors.id)
- status (text) -- pending | shipped | delivered | canceled
- tracking_no (text, nullable)
- carrier (text, nullable)
- created_at, updated_at

## Promotions (Minimal)

### promotions
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- code (text)
- discount_type (text) -- fixed | percent
- value (numeric)
- status (text)
- starts_at (timestamp, nullable)
- ends_at (timestamp, nullable)

## Search / Indexing

Search index is external (Meilisearch). Track update cursors if needed:

### search_sync_state
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- last_product_sync_at (timestamp)

## Multi-Store DB Routing (Optional)

### store_db_routing
- store_id (uuid, pk, fk -> stores.id)
- db_key (text)
- created_at, updated_at

### db_connections
- db_key (text, pk)
- kind (text) -- shared | dedicated
- host (text)
- port (int)
- database_name (text)
- username (text)
- password_secret_ref (text)
- status (text)
- created_at, updated_at

## Indices (Initial)
- products(store_id, status)
- store_locations(store_id, code)
- store_digital_settings(store_id)
- product_digital_settings(product_id)
- variants(product_id, sku)
- inventory_stocks(variant_id, location_id)
- inventory_reservations(expires_at) WHERE status = 'active'
- inventory_reservation_requests(status, is_hot, created_at)
- orders(tenant_id, created_at)
- order_items(order_id)
- digital_deliveries(order_item_id)
- customers(tenant_id, email)
- shipments(order_id, vendor_id)
- carts(tenant_id, created_at)

## Constraints (Initial)
- variants.sku unique per product
- inventory_stocks.stock >= 0
- inventory_stocks.reserved >= 0
- inventory_reservations.quantity > 0
- order_items.quantity > 0
- cart_items.quantity > 0
- digital_deliveries.downloaded_count >= 0

## Notes
- Split orders into vendor shipments for mall mode.
- Payouts are manual/offline for bank transfer/COD in initial phase.
