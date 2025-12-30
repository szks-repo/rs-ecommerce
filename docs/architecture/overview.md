# Architecture (Draft)

## Goals
- Rust-based ecommerce platform
- Mainstream use: single-brand store (hon-ten)
- Optional: mall mode (multi-vendor) without harming single-brand simplicity
- Payments (initial): bank transfer, cash on delivery (COD)
- Fast, flexible search and operations

## Non-Functional Assumptions
- Orders: up to 100,000 transactions per day
- Admin users: up to 500 staff accounts
- Mall: up to 2,000 tenants

## High-Level Components

### StoreBackend
- Domain logic and data management
- Admin APIs (Backoffice)
- Public APIs (Storefront)
- Event/outbox for async processing

### Storefront
- Search and product discovery
- Public shopping flow (cart/checkout/order status)
- Cache/search/index updates driven by events

### Shared Infrastructure
- DB: PostgreSQL
- Cache/Queue: Redis (or Postgres outbox + worker)
- Search: Meilisearch (initially)
- Observability: tracing + structured logs + metrics

## Separation: Backoffice vs Storefront

### Backoffice (Admin)
- Product/Inventory/Price/Promotion management
- Order management and fulfillment
- Customer management
- Role-based access controls
- Bulk import/export, audit trail

### Storefront (Public)
- Fast search and navigation
- Cart/checkout/order tracking
- Caching and CDN-optimized responses

## Domain Boundaries (DDD-style)

- Tenant (Shop)
- Product
- Inventory
- Pricing
- Promotions
- Orders
- Customers
- Fulfillment
- Payments (bank transfer, COD)
- Search (indexing & query)

## Domain ValueObjects

- `src/domain/ids`: typed IDs (`StoreId`, `TenantId`, `CustomerId`, `SkuId`/`VariantId` etc.) instead of raw `String`
- `src/domain/status`: enums for status/type (`ProductStatus`, `VariantStatus`, `FulfillmentType`, etc.)
- `src/domain/validation`: validated values (`StoreCode`, `SkuCode`, `Email`, `Phone`)
- Boundary conversion: RPC/DB layers convert to/from ValueObjects; domain logic prefers typed values

## Multi-Tenant Design (Mainstream = Single Brand)

### Tenancy Strategy (Phase 1)
- Single shared database with `tenant_id` on all core tables
- App-level enforcement (+ optional RLS in Postgres later)
- Each tenant can run in "single-brand mode" by default

### Mall Mode (Optional)
- `mall` can host multiple `vendors`
- Orders can contain items from multiple vendors
- Split order into vendor shipments and vendor payouts

## Data Model (Key Tables)

- tenants
  - id, name, type (single_brand | mall), default_currency, settings
- vendors
  - id, tenant_id, name, commission_rate, status
  - For single-brand, vendor may be the tenant's own store (1:1)
- stores
  - id, tenant_id, name, status
- store_locations
  - id, store_id, code, name, status
- products
  - id, tenant_id, store_id, vendor_id, title, description, status
- product_locations
  - product_id, location_id
- store_digital_settings
  - store_id, default_url_ttl_seconds, default_max_downloads
- product_digital_settings
  - product_id, url_ttl_seconds, max_downloads
- variants
  - id, product_id, sku, fulfillment_type, price_amount, compare_at_amount, status
- inventory_stocks
  - id, tenant_id, store_id, location_id, variant_id, stock, reserved
- inventory_reservations
  - id, tenant_id, store_id, cart_id, variant_id, qty, status, expires_at
- inventory_reservation_requests
  - id, tenant_id, store_id, cart_id, variant_id, qty, status, is_hot
- orders
  - id, tenant_id, customer_id, status, total, payment_method
- order_items
  - id, order_id, vendor_id, variant_id, price, qty
- digital_deliveries
  - id, order_item_id, provider, object_key, url_expires_at, status
- payments
  - id, order_id, method (bank_transfer | cod), status
- shipments
  - id, order_id, vendor_id, status, tracking_no
- customers
  - id, tenant_id, email, name
- storefront_search_index
  - external search index (Meilisearch)

## Order Flow (Single Brand)

1. Storefront creates cart and checkout
2. Order placed -> status = pending_payment (bank transfer) or pending_shipment (COD)
3. Backoffice confirms bank transfer or prepares COD shipment
4. Fulfillment updates order status -> shipped -> completed

## Order Flow (Mall / Multi-Vendor)

1. Storefront creates cart containing items from multiple vendors
2. Order placed -> status = pending_payment or pending_shipment
3. Split order into vendor shipments
4. Backoffice and vendor-specific fulfillment update shipments
5. Payouts handled by platform rules (initially offline/manual)

## Events / Async

### Inventory Reservation Worker
- Add-to-cart enqueues reservation requests.
- Worker processes queue in small batches and releases expired holds.
- Hot items are separated via `is_hot` to avoid blocking other traffic.

### Core Events
- product.updated
- inventory.updated
- price.updated
- order.placed
- inventory.reservation.expired
 - inventory.reservation.requested
- order.status_changed

### Uses
- Update search index
- Cache invalidation
- Send notifications (email/webhook)

## API Strategy

### Public API (Storefront)
- GET /products, /search, /categories
- POST /cart, /checkout
- GET /orders/{id}

### Admin API (Backoffice)
- CRUD /products, /variants, /inventory
- CRUD /orders, /shipments
- CRUD /promotions

## Initial Tech Stack (Rust)
- Web: axum or actix-web
- ORM: sqlx (or sea-orm)
- Migration: sqlx-cli or refinery
- Search: Meilisearch SDK

## Next Steps

1. Confirm API style (REST vs GraphQL)
2. Decide tenancy enforcement (app-only vs RLS)
3. Draft DB schema and migrations
