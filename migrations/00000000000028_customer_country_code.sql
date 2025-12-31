ALTER TABLE customer_profiles
    ADD COLUMN IF NOT EXISTS country_code text NOT NULL DEFAULT 'JP';

ALTER TABLE customer_addresses
    ADD COLUMN IF NOT EXISTS country_code text NOT NULL DEFAULT 'JP';
