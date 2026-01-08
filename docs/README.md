# Docs Index

## Architecture
- `docs/architecture/overview.md`
- `docs/architecture/db_schema.md`
- `docs/architecture/proto_organization.md`
- `docs/architecture/auth_ui.md`
- `docs/architecture/identity.md`
- `docs/architecture/inventory_reservation.md`
- `docs/architecture/inventory_management.md`
- `docs/architecture/multi_store.md`

## API
- `docs/api/connectrpc.md`
- `docs/api/connectrpc_examples.md`
- `docs/api/auth.md`

## Development
- `docs/dev/setup.md`

## Workspace Layout
- `crates/app` (`rs-ecommerce`): API server (main service)
- `crates/cli` (`rs-ecommerce-cli`): operational CLI (search reindex, etc.)
- `crates/common`: shared telemetry/env helpers
- `crates/workers/inventory-worker`: inventory reservation worker
- `crates/workers/customer-sync-worker`: customer sync worker

## Operations
- `docs/operations/audit_log.md`
- `docs/operations/logging.md`

## Features
- Store Settings
  - `docs/features/store_settings/overview.md`
  - `docs/features/store_settings/api.md`
  - `docs/features/store_settings/schema.md`
  - `docs/features/store_settings/form_spec.md`
- Product
  - `docs/features/product/overview.md`
  - `docs/features/product/api.md`
- Order
  - `docs/features/order/overview.md`
  - `docs/features/order/api.md`
- Promotion
  - `docs/features/promotion/overview.md`
  - `docs/features/promotion/api.md`
- Cart
  - `docs/features/cart/overview.md`
  - `docs/features/cart/api.md`
- Setup
  - `docs/features/setup/overview.md`
  - `docs/features/setup/api.md`
  - `docs/features/setup/ui_spec.md`
  - `docs/features/setup/login_ui_spec.md`
