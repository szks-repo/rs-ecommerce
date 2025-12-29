ALTER TABLE stores
    ADD COLUMN IF NOT EXISTS code text;

CREATE UNIQUE INDEX IF NOT EXISTS stores_code_unique
    ON stores (code)
    WHERE code IS NOT NULL;
