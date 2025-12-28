# Init WebUI + Admin/Staff Login Architecture (Draft)

## Goal
- Provide a guided Init WebUI and a Backoffice login for admin/staff.
- Use `shadcn/ui` for UI components.
- Keep backend API (ConnectRPC JSON) as the source of truth.

## High-Level Components
- **Init UI** (one-time setup)
  - Purpose: initial store creation and defaults
  - Calls: `SetupService.InitializeStore`
- **Backoffice UI** (admin/staff)
  - Purpose: manage products, orders, promotions, settings
  - Calls: Backoffice + StoreSettings APIs
- **Identity Service**
  - Issues JWT access tokens
  - Manages admin/staff accounts + roles/permissions
- **API Backend** (current `rs-ecommerce`)
  - ConnectRPC JSON endpoints
  - Actor resolution via JWT
  - Audit logging

## Suggested Frontend Stack
- **Framework**: Next.js (App Router) or Vite + React
- **UI**: `shadcn/ui` + Tailwind CSS
- **State**: React Query or TanStack Query for API calls
- **Auth**: access token in memory + refresh token in httpOnly cookie

## Authentication Flow (Phase 1)
1. User visits Backoffice login page
2. Credentials POST to IdentityService
3. IdentityService returns JWT access token (and optional refresh token)
4. UI stores access token (memory) and attaches `Authorization: Bearer <token>`
5. API backend resolves actor from JWT and writes audit logs

## JWT Claims (proposal)
- `sub`: user_id
- `actor_type`: admin | staff
- `tenant_id`: active tenant
- `roles`: list
- `scopes`: list
- `exp`, `iat`, `iss`, `aud`

## Init WebUI Flow
1. Admin enters store basics (name, address, tax, payments)
2. UI calls `SetupService.InitializeStore`
3. On success, redirect to Login page
4. Init endpoint can be locked after first success

## Data Model (Identity)
- `store_staff`
  - id, store_id, email/login_id/phone, password_hash, role, status
- `store_roles` / `permissions`
  - role definitions and permission mapping

## API Boundaries
- Init UI -> Setup API (public but restricted by one-time token)
- Backoffice UI -> Backoffice + StoreSettings APIs (requires JWT)
- IdentityService -> part of the API backend (shared DB)

## UI Routes (proposal)
- `/init` (Init WebUI)
- `/login` (Backoffice login)
- `/admin/*` (Backoffice app)

## Security Notes
- Disable Init after first successful setup (or require admin invite token)
- Require TLS in production
- Use audit logs for all settings changes

## Rollout
- Phase 0: Init UI only (no auth)
- Phase 1: Add IdentityService + login
- Phase 2: Roles/scopes + tenant switching
