# Proto Organization (Draft)

Goal: keep context boundaries explicit and allow growth without cross-contamination.

## Proposed Units (per bounded context)
- `common.proto`
  - shared primitives (TenantContext, Money, PageInfo, enums that are truly global)
- `storefront.proto`
  - StorefrontService (read/search/cart/checkout)
- `backoffice.proto`
  - BackofficeService (product/order/fulfillment/admin ops)
- `store_settings.proto`
  - StoreSettingsService (store configuration, shipping/tax/mall)
- `setup.proto`
  - SetupService (initial setup / bootstrap flow)

## Guidelines
- Each proto file maps to **one bounded context or access layer**.
- Avoid cross-context messages unless moved into `common.proto`.
- Only shared enums belong in `common.proto`.
- If a message is used by two contexts, extract to `common.proto` or duplicate explicitly.

## Future Split Candidates
- `product.proto` (Product context)
- `order.proto` (Order/Fulfillment context)
- `customer.proto` (Customer context)
- `search.proto` (Search/indexing context)

## Versioning
- Keep package as `ecommerce.v1`
- Add `ecommerce.v2` when breaking changes accumulate
