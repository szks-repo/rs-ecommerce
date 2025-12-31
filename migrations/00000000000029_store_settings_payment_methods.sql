ALTER TABLE store_settings
ADD COLUMN IF NOT EXISTS bank_transfer_enabled bool NOT NULL DEFAULT true;
