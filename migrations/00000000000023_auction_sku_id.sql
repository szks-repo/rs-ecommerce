ALTER TABLE auctions
    RENAME COLUMN variant_id TO sku_id;

DROP INDEX IF EXISTS auctions_store_status_idx;
CREATE INDEX IF NOT EXISTS auctions_store_status_idx
    ON auctions (store_id, status, end_at);
