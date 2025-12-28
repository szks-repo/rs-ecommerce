-- Catalog/Inventory v2 (store-scoped) + async reservation queue.
-- Note: This migration replaces the original draft catalog tables (no data migration).

DROP TABLE IF EXISTS inventory_reservation_requests;
DROP TABLE IF EXISTS inventory_reservations;
DROP TABLE IF EXISTS order_items;
DROP TABLE IF EXISTS cart_items;
DROP TABLE IF EXISTS inventory_items;
DROP TABLE IF EXISTS variants;
DROP TABLE IF EXISTS products;

CREATE TABLE IF NOT EXISTS products (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    vendor_id uuid REFERENCES vendors(id),
    title text NOT NULL,
    description text NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS variants (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id uuid NOT NULL REFERENCES products(id),
    sku text NOT NULL,
    price_amount bigint NOT NULL,
    price_currency text NOT NULL,
    compare_at_amount bigint,
    compare_at_currency text,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS variants_product_sku_idx
    ON variants (product_id, sku);

CREATE INDEX IF NOT EXISTS products_store_status_idx
    ON products (store_id, status);

CREATE TABLE IF NOT EXISTS inventory_stocks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    variant_id uuid NOT NULL REFERENCES variants(id),
    stock int NOT NULL,
    reserved int NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (variant_id)
);

-- Reservation records (time-bound holds).
CREATE TABLE IF NOT EXISTS inventory_reservations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    cart_id uuid NOT NULL REFERENCES carts(id),
    cart_item_id uuid REFERENCES cart_items(id),
    variant_id uuid NOT NULL REFERENCES variants(id),
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

CREATE INDEX IF NOT EXISTS inventory_reservations_variant_idx
    ON inventory_reservations (variant_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS inventory_reservations_cart_item_active_idx
    ON inventory_reservations (cart_item_id)
    WHERE status = 'active' AND cart_item_id IS NOT NULL;

-- Async reservation queue (hot-item isolation).
CREATE TABLE IF NOT EXISTS inventory_reservation_requests (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    cart_id uuid NOT NULL REFERENCES carts(id),
    cart_item_id uuid REFERENCES cart_items(id),
    variant_id uuid NOT NULL REFERENCES variants(id),
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

CREATE INDEX IF NOT EXISTS inventory_reservation_requests_tenant_idx
    ON inventory_reservation_requests (tenant_id, store_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS inventory_reservation_requests_idem_idx
    ON inventory_reservation_requests (idempotency_key)
    WHERE idempotency_key IS NOT NULL;

-- Recreate cart_items/order_items with new variant FK (no data migration).
CREATE TABLE IF NOT EXISTS cart_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id uuid NOT NULL REFERENCES carts(id),
    vendor_id uuid REFERENCES vendors(id),
    variant_id uuid NOT NULL REFERENCES variants(id),
    price_amount bigint NOT NULL,
    price_currency text NOT NULL,
    quantity int NOT NULL
);

CREATE TABLE IF NOT EXISTS order_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id uuid NOT NULL REFERENCES orders(id),
    vendor_id uuid REFERENCES vendors(id),
    variant_id uuid NOT NULL REFERENCES variants(id),
    price_amount bigint NOT NULL,
    price_currency text NOT NULL,
    quantity int NOT NULL
);
