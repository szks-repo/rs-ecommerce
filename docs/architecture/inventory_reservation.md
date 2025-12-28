# Inventory Reservation Architecture (Draft)

## Goal
- Reserve inventory when a customer adds items to cart.
- Release holds automatically if the cart is not checked out within a TTL.
- Prevent hot items from blocking other items or tenants.

## Reservation Timing
- Reservation is created at **add-to-cart** time.
- Each reservation has `expires_at` (TTL) and `status`.
- On checkout, reservations are **consumed** (stock decreases, reserved decreases).
- On expiry or cart removal, reservations are **released** (reserved decreases).

## Data Model (current)
- `inventory_stocks`
  - `stock` and `reserved` counters
- `inventory_reservations`
  - time-bound holds linked to cart + variant

## Workflow (core)
1) Add-to-cart:
   - Start a transaction.
   - Insert `inventory_reservations` with status=`active` and TTL.
   - Atomically update `inventory_stocks`:
     - `reserved = reserved + qty` only if `stock - reserved >= qty`.
   - If update fails, rollback and return out-of-stock.
   - Digital variants skip reservation and inventory updates.

2) Release (expiry or cart item removal):
   - Background worker selects expired `active` reservations using `FOR UPDATE SKIP LOCKED`.
   - Decrement `inventory_items.reserved` and mark reservation `expired`.

3) Consume (checkout):
   - In a transaction, lock active reservations for the cart.
   - Decrement `stock` and `reserved`, mark reservation `consumed`.

## Async / Reservation Queue
- Cart add requests are queued and processed asynchronously to smooth spikes.
- Queue is split for hot vs normal items to isolate hotspots.
- Status table tracks request state and idempotency for at-least-once delivery.
- Workers consume in small batches and use `SKIP LOCKED` where applicable.

## Runtime (PaaS assumption)
- We will run the worker as a **Cloud Run Job** (GCP).
- The job processes a bounded batch and exits (`INVENTORY_WORKER_ONESHOT=true`).
- Trigger cadence: every 1-2 minutes via Cloud Scheduler.

### Request Status (suggested)
- `queued` -> `processing` -> `done`
- `queued` -> `processing` -> `failed` (retry with backoff)

### Reservation Status (suggested)
- `active` -> `consumed` (checkout)
- `active` -> `expired` (TTL)
- `active` -> `released` (cart remove)

### Worker Loop (suggested)
1. Claim N requests by status=`queued` with `FOR UPDATE SKIP LOCKED`.
2. Mark `processing` + set `updated_at`.
3. Try to reserve stock:
   - `UPDATE inventory_stocks SET reserved = reserved + qty`
     `WHERE variant_id = $1 AND stock - reserved >= qty`.
4. If success:
   - Insert `inventory_reservations` (active, expires_at).
   - Mark request `done`.
5. If failed (out of stock):
   - Mark request `failed` with reason.

### API/User Feedback (suggested)
- Add-to-cart/Update-cart returns immediately with `reservation_status = queued`.
- Client can poll a reservation status endpoint (future) or observe failure events.
- If a reservation fails, UI should surface "out of stock" and revert quantity.

### Expiry Releaser (suggested)
1. Select `inventory_reservations` where `expires_at < now()` and status=`active`.
2. `FOR UPDATE SKIP LOCKED` in small batches.
3. Decrement `inventory_stocks.reserved`.
4. Mark reservation `expired`.

## Hot Item / Tenant Isolation
- Split the reservation queue for hot items to avoid blocking other traffic.
- Keep updates scoped per-variant row (no global locks).
- Optional sharding:
  - partition workers by tenant_id or hash(variant_id) to distribute hot variants.

## TTL Notes
- TTL should be configurable (e.g., 15-30 minutes).
- Business choice; no hard-coded assumption.

## ZOZO Reference (migration architecture)
- ZOZOTOWN moved cart add to a queued architecture to reduce DB load under spikes.
- A status table + queue separates popular items and enables idempotent processing.
- This informs our async reservation queue + hot-item isolation design.
