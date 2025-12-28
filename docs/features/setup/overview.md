# Feature: Setup

## Purpose
- One-time initialization for a new store (tenant + store + owner staff).

## Scope
- Included:
  - InitializeStore (setup API)
- Excluded:
  - Store settings (configured post-init)
  - Guided onboarding UI (future)

## Domain Model (draft)
- Entities:
  - Tenant
  - Store
  - Vendor
  - StoreStaff (owner)

## APIs
- SetupService.InitializeStore

## Data Model
- Tables:
  - tenants
  - stores
  - vendors
  - store_staff

## Flows
- Initialize store:
  1. Validate inputs
  2. Insert tenant/store/vendor
  3. Insert owner staff

## Audit
- Actions:
  - (future) setup.initialize

## Open Questions
- Idempotency strategy for repeated calls
