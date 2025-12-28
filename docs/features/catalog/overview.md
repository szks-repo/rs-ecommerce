# Feature: Catalog

## Purpose
- Manage products, variants, and inventory for storefront and backoffice.

## Scope
- Included:
  - Product CRUD (backoffice)
  - Variant CRUD (backoffice)
  - Inventory set/update
  - Search index sync (Meilisearch)
- Excluded:
  - Advanced taxonomy (categories/collections) (future)
  - Media asset management (future)

## Domain Model (draft)
- Entities:
  - Product
  - Variant
  - InventoryItem
- Invariants:
  - Variant belongs to a product
  - Inventory is per variant

## APIs
- BackofficeService.CreateProduct / UpdateProduct
- BackofficeService.CreateVariant / UpdateVariant
- BackofficeService.SetInventory
- StorefrontService.ListProducts / GetProduct / SearchProducts

## Data Model
- Tables:
  - products
  - variants
  - inventory_items

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
