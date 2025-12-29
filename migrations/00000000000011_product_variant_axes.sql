CREATE TABLE IF NOT EXISTS product_variant_axes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id uuid NOT NULL REFERENCES products(id),
    name text NOT NULL,
    position int NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (product_id, name)
);

CREATE INDEX IF NOT EXISTS product_variant_axes_product_idx
    ON product_variant_axes (product_id, position);
