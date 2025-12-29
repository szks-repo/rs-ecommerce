ALTER TABLE products
    ADD COLUMN IF NOT EXISTS tax_rule_id uuid REFERENCES tax_rules(id);

CREATE INDEX IF NOT EXISTS products_tax_rule_id_idx
    ON products (tax_rule_id);
