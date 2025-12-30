ALTER TABLE store_settings
    ADD COLUMN IF NOT EXISTS time_zone text NOT NULL DEFAULT 'Asia/Tokyo';
