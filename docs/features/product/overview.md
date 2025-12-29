# Feature: Product

## Purpose
- Manage products, variants, and inventory for storefront and backoffice.

## Scope
- Included:
  - Product CRUD (backoffice)
  - Variant CRUD (backoffice)
  - Inventory set/update (per location)
  - Search index sync (Meilisearch)
- Excluded:
  - Advanced taxonomy (categories/collections) (future)
  - Media asset management (future)

## Domain Model (draft)
- Entities:
  - Product
  - Variant (SKU)
  - InventoryStock
  - StoreLocation
- Invariants:
  - Variant belongs to a product
  - Inventory is per variant + location (physical only)
  - Product can be linked to one or more store locations
  - Digital variants do not require inventory rows
  - Product is scoped to a store

## APIs
- BackofficeService.CreateProduct / UpdateProduct
- BackofficeService.CreateVariant / UpdateVariant
- BackofficeService.SetInventory
- StorefrontService.ListProducts / GetProduct / SearchProducts

## Data Model
- Tables:
  - products
  - product_locations
  - store_locations
  - variants
  - inventory_stocks

## Flows
- Product create/update:
  1. Write DB
  2. Upsert search index

## Audit
- Actions:
  - product.create
  - product.update
  - variant.create
  - variant.update
  - inventory.set

## Open Questions
- Category/collection model
- Product media storage

## Decisions (Draft)
- Product can be created in two modes:
  - Single-SKU product: auto-create a default Variant (SKU) on product creation.
  - Multi-variant product: define variant axes (e.g. size/color) first, then create variants.
- Jan code is optional and belongs to Variant (SKU).
- Product can hold listing/sale windows and product-level sale pricing.
- Cross-product campaigns remain in Promotion (later).
