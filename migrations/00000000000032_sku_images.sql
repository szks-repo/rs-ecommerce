CREATE TABLE IF NOT EXISTS store_media_assets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    provider text NOT NULL DEFAULT '',
    bucket text NOT NULL DEFAULT '',
    object_key text NOT NULL DEFAULT '',
    public_url text NOT NULL,
    content_type text,
    size_bytes bigint,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_media_assets_store_idx
    ON store_media_assets (store_id, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS store_media_assets_store_key_idx
    ON store_media_assets (store_id, object_key)
    WHERE object_key <> '';

CREATE TABLE IF NOT EXISTS sku_images (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    sku_id uuid NOT NULL REFERENCES variants(id),
    asset_id uuid NOT NULL REFERENCES store_media_assets(id),
    position int NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (sku_id, asset_id),
    UNIQUE (sku_id, position)
);

CREATE INDEX IF NOT EXISTS sku_images_sku_idx ON sku_images (sku_id);
