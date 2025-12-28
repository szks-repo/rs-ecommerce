# Feature: Cart

## Purpose
- Manage cart lifecycle and line items for checkout.

## Scope
- Included:
  - Create cart
  - Add/update/remove items
  - Inventory reservation on add-to-cart (TTL hold)
- Excluded:
  - Price recalculation rules (future)
  - Cart expiration policy (future)

## Domain Model (draft)
- Entities:
  - Cart
  - CartItem

## APIs
- StorefrontService.CreateCart
- StorefrontService.AddCartItem / UpdateCartItem / RemoveCartItem

## Data Model
- Tables:
  - carts
  - cart_items
  - inventory_reservations
  - inventory_items (reserved counter)

## Flows
- Add item:
  1. Validate variant
  2. Create inventory reservation (time-bound)
  3. Update cart totals
  4. Release reservation on expiry or remove

## Audit
- Actions:
  - (not yet tracked)

## Open Questions
- Price snapshot strategy
- Guest cart merge
