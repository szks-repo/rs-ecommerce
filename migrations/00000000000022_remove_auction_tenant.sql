ALTER TABLE auctions
    DROP COLUMN IF EXISTS tenant_id;

ALTER TABLE auction_bids
    DROP COLUMN IF EXISTS tenant_id;

DROP INDEX IF EXISTS auctions_store_status_idx;
CREATE INDEX IF NOT EXISTS auctions_store_status_idx
    ON auctions (store_id, status, end_at);

DROP INDEX IF EXISTS auction_bids_auction_idx;
CREATE INDEX IF NOT EXISTS auction_bids_auction_idx
    ON auction_bids (auction_id, created_at);
