ALTER TABLE store_settings
ADD COLUMN IF NOT EXISTS storage_provider text NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS storage_bucket text NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS storage_base_path text NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS storage_cdn_base_url text NOT NULL DEFAULT '';
