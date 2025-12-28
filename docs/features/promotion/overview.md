# Feature: Promotion

## Purpose
- Define discounts and promo codes for orders.

## Scope
- Included:
  - Promotion create/update (backoffice)
- Excluded:
  - Auto-apply rules (future)
  - Stacking/priority (future)

## Domain Model (draft)
- Entities:
  - Promotion
- Invariants:
  - Code is unique per tenant (future constraint)

## APIs
- BackofficeService.CreatePromotion / UpdatePromotion

## Data Model
- Tables:
  - promotions

## Flows
- Promotion create:
  1. Validate
  2. Persist

## Audit
- Actions:
  - promotion.create
  - promotion.update

## Open Questions
- Validation rules for date overlap
- Code uniqueness enforcement
