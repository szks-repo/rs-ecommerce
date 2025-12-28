# Auth & Actor Resolution (Draft)

## Goal
Resolve `ActorContext` (actor_id / actor_type) from an authenticated token, and attach it to each request for audit logging and authorization decisions.

## Token Format (proposal)
JWT access token signed by auth service.
- Standard claims: `iss`, `aud`, `sub`, `exp`, `iat`
- Custom claims:
  - `actor_type` (admin | staff | api | system)
  - `tenant_id`
  - `vendor_id` (optional for mall vendors)
  - `roles` (list)
  - `scopes` (list)

## Verification Flow (proposal)
1. Read `Authorization: Bearer <token>` from headers.
2. Verify JWT signature using JWKs published by auth service:
   - `AUTH_JWKS_URL` (HTTPS)
   - Cache keys with TTL (e.g., 5 minutes).
3. Validate claims:
   - `iss` matches expected issuer
   - `aud` includes this API
   - `exp` not expired
4. Map to `ActorContext`:
   - `actor_id = sub`
   - `actor_type = actor_type` (fallback to `api`)
5. Attach to request extensions for handlers to consume.

## Precedence Rules (proposal)
1. If `Authorization` is present and valid, use it.
2. If `Authorization` is missing, allow `x-actor-id` / `x-actor-type` for local/dev only.
3. Body `actor` should be ignored when `Authorization` is present to prevent spoofing.

## Implementation Notes
- Keep middleware in `src/rpc/actor.rs`.
- Replace the temporary `actor_from_bearer` logic with JWT verification.
- Enforce tenant scoping using `tenant_id` claim in later authorization layer.

## Non-Goals (now)
- User login UI or session cookies
- Permission matrix enforcement (roles/scopes used later)
