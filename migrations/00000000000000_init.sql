-- Initial schema for core ecommerce entities.
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS tenants (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    type text NOT NULL,
    default_currency text NOT NULL,
    status text NOT NULL,
    settings jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS vendors (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    name text NOT NULL,
    commission_rate numeric,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);


CREATE TABLE IF NOT EXISTS customers (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    email text NOT NULL,
    name text NOT NULL,
    phone text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS carts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    customer_id uuid REFERENCES customers(id),
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS cart_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id uuid NOT NULL REFERENCES carts(id),
    vendor_id uuid REFERENCES vendors(id),
    variant_id uuid NOT NULL REFERENCES variants(id),
    price_amount bigint NOT NULL,
    price_currency text NOT NULL,
    quantity int NOT NULL
);

CREATE TABLE IF NOT EXISTS orders (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    customer_id uuid REFERENCES customers(id),
    status text NOT NULL,
    total_amount bigint NOT NULL,
    currency text NOT NULL,
    payment_method text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
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

CREATE TABLE IF NOT EXISTS shipments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id uuid NOT NULL REFERENCES orders(id),
    vendor_id uuid REFERENCES vendors(id),
    status text NOT NULL,
    tracking_no text,
    carrier text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS promotions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    code text NOT NULL,
    discount_type text NOT NULL,
    value_amount bigint NOT NULL,
    value_currency text NOT NULL,
    status text NOT NULL,
    starts_at timestamptz,
    ends_at timestamptz
);

CREATE INDEX IF NOT EXISTS orders_tenant_created_idx ON orders (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS customers_tenant_email_idx ON customers (tenant_id, email);
