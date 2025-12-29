# Customers (Design Draft)

This document defines the customer domain with cross-store identity resolution.
The goal is to manage customers per tenant while allowing store-specific profiles.

## Goals
- Identify the same person across stores under one tenant.
- Keep store-specific data (name, preferences, notes) isolated per store.
- Support optional identifiers (email / phone / external IDs).

## Core Concepts
- **Canonical Customer (tenant-level)**: a durable identity root.
- **Customer Profile (store-level)**: store-specific view (name, status, notes).
- **Customer Identity (identity map)**: normalized identifiers for matching.

## Matching / Resolution
1) Normalize identifiers (email: lowercase/trim, phone: digits only).
2) Look up `customer_identities` by `(tenant_id, identity_type, identity_value)`.
3) If match exists, reuse the canonical customer.
4) If multiple matches, prefer verified identities; otherwise manual review (future).
5) If no match, create a new canonical customer + profile + identities.

## Minimal Data Model
### customers (canonical)
- id (uuid, pk)
- tenant_id (uuid, fk)
- status (text)
- created_at, updated_at

### customer_profiles (store-level)
- id (uuid, pk)
- customer_id (uuid, fk)
- store_id (uuid, fk)
- name (text)
- email (text, nullable)
- phone (text, nullable)
- status (text)
- notes (text, nullable)
- preferences (jsonb)
- created_at, updated_at

### customer_identities (identity map)
- id (uuid, pk)
- tenant_id (uuid, fk)
- customer_id (uuid, fk)
- identity_type (text) -- email | phone | external
- identity_value (text, normalized)
- verified (bool)
- source (text) -- signup | import | merge | admin
- created_at

### customer_addresses
- id (uuid, pk)
- customer_id (uuid, fk)
- type (text) -- shipping | billing
- name (text)
- postal_code (text)
- prefecture (text)
- city (text)
- line1 (text)
- line2 (text, nullable)
- phone (text, nullable)
- created_at, updated_at

## API Outline (Admin)
- `ListCustomers` (store-scoped, search)
- `GetCustomer` (detail, identities + addresses)
- `CreateCustomer` (resolves identity, creates profile)
- `UpdateCustomer` (profile updates, status changes)

## Notes
- Email/phone are optional; identity records drive same-person matching.
- Store profile is required to show a customer in a store context.
- All merges and identity changes should be audited.
