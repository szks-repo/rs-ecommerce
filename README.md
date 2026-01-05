# rs-ecommerce

Rust-based e-commerce platform (storefront + admin + operations).

## Status
This project is **in planning + active implementation** and **not yet pre-alpha**.  
Specs, data models, and APIs are still shifting; breaking changes and incomplete features are expected.

## Features (current / planned)
- Store setup and settings (basic info, payment, tax, shipping, storage)
- Product catalog with SKU/variant support
- Inventory management
- Customer management
- Staff/role/permission management
- Audit logs
- Auctions (draft → publish, bidding)
- Metafield definitions and values

## Search reindex (Meilisearch)
Reindex command:
```bash
cargo run --bin search_reindex
```

Search backend selection:
- `SEARCH_BACKEND` (default: `meili`; allowed: `meili`, `opensearch`, `none`)
- `MEILI_URL` (required when `SEARCH_BACKEND=meili`)
- `MEILI_MASTER_KEY` (optional)
- `OPENSEARCH_URL` (required when `SEARCH_BACKEND=opensearch`)
- `OPENSEARCH_INDEX` (default: `products`)
  - NOTE: OpenSearch backend is currently a noop (startup warning + no indexing/search). Use `meili` for active search.

Environment variables:
- `DATABASE_URL` (required)
- `MEILI_INDEX` (default: `products`)
- `REINDEX_BATCH_SIZE` (default: `500`)
- `REINDEX_DRY_RUN` (`1`/`true` to skip writes)
- `REINDEX_COUNT_ONLY` (`1`/`true` to only count and exit)
- `REINDEX_TENANT_ID` (optional filter)
- `REINDEX_STORE_ID` (optional filter)
- `REINDEX_VENDOR_ID` (optional filter)
- `REINDEX_STATUS` (optional filter)
- `REINDEX_PRODUCT_ID` (optional filter; reindex single product)

## Documentation
See `docs/README.md` for the documentation index.
