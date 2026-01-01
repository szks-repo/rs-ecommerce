ALTER TABLE store_storage_settings
    ADD COLUMN IF NOT EXISTS public_storage_provider text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS public_storage_bucket text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS public_storage_base_path text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS public_storage_cdn_base_url text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS public_storage_region text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS private_storage_provider text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS private_storage_bucket text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS private_storage_base_path text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS private_storage_cdn_base_url text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS private_storage_region text NOT NULL DEFAULT '';

UPDATE store_storage_settings
SET public_storage_provider = storage_provider,
    public_storage_bucket = storage_bucket,
    public_storage_base_path = storage_base_path,
    public_storage_cdn_base_url = storage_cdn_base_url
WHERE public_storage_provider = '' AND public_storage_bucket = '';

ALTER TABLE store_storage_settings
    DROP COLUMN IF EXISTS storage_provider,
    DROP COLUMN IF EXISTS storage_bucket,
    DROP COLUMN IF EXISTS storage_base_path,
    DROP COLUMN IF EXISTS storage_cdn_base_url;
