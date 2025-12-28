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

## Catalog

### products
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- vendor_id (uuid, fk -> vendors.id)
- title (text)
- description (text)
- status (text) -- draft | active | archived
- created_at, updated_at

### product_options
- id (uuid, pk)
- product_id (uuid, fk -> products.id)
- name (text)
- position (int)

### product_option_values
- id (uuid, pk)
- product_option_id (uuid, fk -> product_options.id)
- value (text)
- position (int)

### variants
- id (uuid, pk)
- product_id (uuid, fk -> products.id)
- sku (text, unique per tenant)
- price (numeric)
- compare_at (numeric, nullable)
- weight (numeric, nullable)
- status (text)
- created_at, updated_at

### variant_option_values
- id (uuid, pk)
- variant_id (uuid, fk -> variants.id)
- product_option_value_id (uuid, fk -> product_option_values.id)

## Inventory

### inventory_items
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- variant_id (uuid, fk -> variants.id)
- stock (int)
- reserved (int)
- updated_at

## Customers

### customers
- id (uuid, pk)
- tenant_id (uuid, fk -> tenants.id)
- email (text)
- name (text)
- phone (text, nullable)
- created_at, updated_at

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
- price (numeric)
- quantity (int)

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

## Indices (Initial)
- products(tenant_id, status)
- variants(product_id)
- inventory_items(variant_id)
- orders(tenant_id, created_at)
- order_items(order_id)
- customers(tenant_id, email)
- shipments(order_id, vendor_id)
- carts(tenant_id, created_at)

## Constraints (Initial)
- variants.sku unique per tenant
- inventory_items.stock >= 0
- inventory_items.reserved >= 0
- order_items.quantity > 0
- cart_items.quantity > 0

## Notes
- Split orders into vendor shipments for mall mode.
- Payouts are manual/offline for bank transfer/COD in initial phase.
