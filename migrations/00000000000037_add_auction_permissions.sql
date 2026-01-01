INSERT INTO permissions (key, name, description) VALUES
    ('auction.read', 'Auction Read', 'Read auctions and bids'),
    ('auction.write', 'Auction Write', 'Create/update auctions and place bids')
ON CONFLICT (key) DO NOTHING;
