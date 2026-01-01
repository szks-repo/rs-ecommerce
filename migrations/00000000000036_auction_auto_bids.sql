CREATE TABLE IF NOT EXISTS auction_auto_bids (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    auction_id uuid NOT NULL REFERENCES auctions(id) ON DELETE CASCADE,
    store_id uuid NOT NULL REFERENCES stores(id),
    customer_id uuid NOT NULL REFERENCES customers(id),
    max_amount bigint NOT NULL,
    currency text NOT NULL,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS auction_auto_bids_auction_customer_idx
    ON auction_auto_bids (auction_id, customer_id);

CREATE INDEX IF NOT EXISTS auction_auto_bids_auction_idx
    ON auction_auto_bids (auction_id, status, max_amount DESC, created_at ASC);
