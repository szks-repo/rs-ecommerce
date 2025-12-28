# Feature: Cart

## Purpose
- Manage cart lifecycle and line items for checkout.

## Scope
- Included:
  - Create cart
  - Add/update/remove items
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

## Flows
- Add item:
  1. Validate variant
  2. Update cart totals

## Audit
- Actions:
  - (not yet tracked)

## Open Questions
- Price snapshot strategy
- Guest cart merge
