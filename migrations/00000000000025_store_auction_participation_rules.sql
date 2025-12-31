CREATE TABLE IF NOT EXISTS store_auction_participation_rules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    rule_type text NOT NULL,
    rule_config jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_auction_participation_rules_store_idx
    ON store_auction_participation_rules (store_id, status);
CREATE INDEX IF NOT EXISTS store_auction_participation_rules_type_idx
    ON store_auction_participation_rules (store_id, rule_type);
