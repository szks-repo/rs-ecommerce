# Identity Auth & Actor Resolution (Draft)

## Goal
Resolve `ActorContext` (actor_id / actor_type) from an authenticated token, and attach it to each request for audit logging and authorization decisions.

## Token Format (proposal)
JWT access token signed by IdentityService.
- Standard claims: `iss`, `aud`, `sub`, `exp`, `iat`
- Custom claims:
  - `actor_type` (owner | admin | staff | api | system)
  - `tenant_id`
  - `store_id`
  - `vendor_id` (optional for mall vendors)
  - `roles` (list)
  - `scopes` (list)

## Verification Flow (current direction)
1. Read `Authorization: Bearer <token>` from headers.
2. Verify JWT signature using IdentityService keys:
   - HS256: `AUTH_JWT_SECRET`
   - RS256 (optional): `AUTH_JWKS_URL` (HTTPS) + `AUTH_JWT_ISSUER` / `AUTH_JWT_AUDIENCE`
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
- Replace the temporary `actor_from_bearer` logic with strict JWT verification.
- Enforce tenant scoping using `tenant_id` claim in later authorization layer.

## Non-Goals (now)
- Permission matrix enforcement (roles/scopes used later)
