CREATE TABLE IF NOT EXISTS store_digital_assets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    sku_id uuid NOT NULL REFERENCES variants(id),
    provider text NOT NULL,
    bucket text NOT NULL,
    object_key text NOT NULL,
    content_type text,
    size_bytes bigint,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_digital_assets_store_sku_idx
    ON store_digital_assets (store_id, sku_id, created_at);
