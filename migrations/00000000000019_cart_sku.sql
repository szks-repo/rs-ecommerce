-- Update cart/inventory schema to use store_id + sku_id (no data migration).

ALTER TABLE carts ADD COLUMN IF NOT EXISTS store_id uuid REFERENCES stores(id);
ALTER TABLE carts ADD COLUMN IF NOT EXISTS expires_at timestamptz NOT NULL DEFAULT now() + interval '30 days';
ALTER TABLE carts ALTER COLUMN status SET DEFAULT 'active';
ALTER TABLE carts DROP COLUMN IF EXISTS tenant_id;

DROP TABLE IF EXISTS cart_items CASCADE;
DROP TABLE IF EXISTS inventory_reservations;
DROP TABLE IF EXISTS inventory_reservation_requests;

CREATE TABLE IF NOT EXISTS cart_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id uuid NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    sku_id uuid NOT NULL REFERENCES variants(id),
    location_id uuid REFERENCES store_locations(id),
    unit_price_amount bigint NOT NULL,
    unit_price_currency text NOT NULL,
    quantity int NOT NULL,
    fulfillment_type text NOT NULL,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS cart_items_cart_idx ON cart_items (cart_id);
CREATE INDEX IF NOT EXISTS cart_items_sku_idx ON cart_items (sku_id);

CREATE TABLE IF NOT EXISTS inventory_reservations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    cart_id uuid NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    cart_item_id uuid REFERENCES cart_items(id) ON DELETE CASCADE,
    sku_id uuid NOT NULL REFERENCES variants(id),
    location_id uuid REFERENCES store_locations(id),
    quantity int NOT NULL,
    status text NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (quantity > 0)
);

CREATE INDEX IF NOT EXISTS inventory_reservations_expiry_idx
    ON inventory_reservations (expires_at)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS inventory_reservations_cart_idx
    ON inventory_reservations (cart_id, status);

CREATE INDEX IF NOT EXISTS inventory_reservations_sku_idx
    ON inventory_reservations (sku_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS inventory_reservations_cart_item_active_idx
    ON inventory_reservations (cart_item_id)
    WHERE status = 'active' AND cart_item_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS inventory_reservation_requests (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    cart_id uuid NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    cart_item_id uuid REFERENCES cart_items(id) ON DELETE CASCADE,
    sku_id uuid NOT NULL REFERENCES variants(id),
    location_id uuid REFERENCES store_locations(id),
    quantity int NOT NULL,
    status text NOT NULL,
    is_hot boolean NOT NULL DEFAULT false,
    idempotency_key text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (quantity > 0)
);

CREATE INDEX IF NOT EXISTS inventory_reservation_requests_status_idx
    ON inventory_reservation_requests (status, is_hot, created_at);

CREATE INDEX IF NOT EXISTS inventory_reservation_requests_store_idx
    ON inventory_reservation_requests (store_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS inventory_reservation_requests_idem_idx
    ON inventory_reservation_requests (idempotency_key)
    WHERE idempotency_key IS NOT NULL;
