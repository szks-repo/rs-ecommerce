# Feature: Setup

## Purpose
- One-time initialization for a new store (tenant + vendor + settings + defaults).

## Scope
- Included:
  - InitializeStore (setup API)
  - Default shipping zone/rate/tax rule
- Excluded:
  - Guided onboarding UI (future)

## Domain Model (draft)
- Entities:
  - Tenant
  - Vendor
  - StoreSettings
  - MallSettings

## APIs
- SetupService.InitializeStore

## Data Model
- Tables:
  - tenants
  - vendors
  - store_settings
  - mall_settings
  - shipping_zones, shipping_rates, tax_rules (optional defaults)

## Flows
- Initialize store:
  1. Validate inputs
  2. Insert tenant/vendor
  3. Insert settings + defaults

## Audit
- Actions:
  - store_settings.initialize (via settings update hook)
  - mall_settings.initialize

## Open Questions
- Idempotency strategy for repeated calls
