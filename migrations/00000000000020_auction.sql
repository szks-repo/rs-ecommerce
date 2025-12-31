CREATE TABLE IF NOT EXISTS store_auction_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    bid_increment_amount bigint NOT NULL DEFAULT 100,
    bid_increment_currency text NOT NULL DEFAULT 'JPY',
    fee_rate_percent numeric(5,2) NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auctions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    product_id uuid REFERENCES products(id),
    variant_id uuid REFERENCES variants(id),
    auction_type text NOT NULL,
    status text NOT NULL,
    start_at timestamptz NOT NULL,
    end_at timestamptz NOT NULL,
    bid_increment_amount bigint NOT NULL,
    bid_increment_currency text NOT NULL,
    start_price_amount bigint NOT NULL,
    start_price_currency text NOT NULL,
    reserve_price_amount bigint,
    reserve_price_currency text,
    buyout_price_amount bigint,
    buyout_price_currency text,
    current_price_amount bigint,
    current_price_currency text,
    current_bid_id uuid,
    winning_bid_id uuid,
    winning_price_amount bigint,
    winning_price_currency text,
    approved_by uuid,
    approved_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS auctions_store_status_idx
    ON auctions (store_id, status, end_at);

CREATE TABLE IF NOT EXISTS auction_bids (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    auction_id uuid NOT NULL REFERENCES auctions(id) ON DELETE CASCADE,
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    customer_id uuid REFERENCES customers(id),
    amount bigint NOT NULL,
    currency text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS auction_bids_auction_idx
    ON auction_bids (auction_id, created_at);

INSERT INTO permissions (key, name, description) VALUES
    ('auction.read', 'Auction Read', 'Read auctions and bids'),
    ('auction.write', 'Auction Write', 'Create/update auctions and place bids')
ON CONFLICT (key) DO NOTHING;
