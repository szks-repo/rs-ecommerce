# Audit Log Architecture (Draft)

## Goals
- Immutable trail for admin actions and critical system events
- Queryable by tenant, actor, entity, time
- Low write overhead and append-only semantics

## Scope
- Backoffice operations (catalog/order/shipping/promotion/store_settings)
- Setup/initialization events
- Auth/admin role changes (future)

## Data Model
Append-only table with JSON payload:
- tenant_id
- actor_id (admin/staff)
- actor_type (admin/staff/system)
- action (string: e.g., "store_settings.update")
- target_type (string: e.g., "product", "order", "store_settings")
- target_id (string)
- request_id (trace id)
- ip_address, user_agent (optional)
- before_json / after_json (optional)
- metadata_json (optional)
- created_at

## Write Path
- Synchronous write for critical operations (store_settings, setup)
- Async/eventual for high-volume actions (catalog updates)

## Retention
- Default: 2 years
- Export (CSV/JSON) for compliance

## Indexing
- (tenant_id, created_at)
- (tenant_id, target_type, target_id)
- (tenant_id, actor_id)

## Future
- Partition by month
- Outbox for streaming to external SIEM
