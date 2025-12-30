ALTER TABLE store_staff
    ADD COLUMN IF NOT EXISTS display_name text;

CREATE TABLE IF NOT EXISTS store_staff_invites (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    email text NOT NULL,
    role_id uuid NOT NULL REFERENCES store_roles(id),
    token text NOT NULL,
    created_by uuid REFERENCES store_staff(id),
    expires_at timestamptz NOT NULL,
    accepted_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS store_staff_invites_token_unique
    ON store_staff_invites (token);

CREATE UNIQUE INDEX IF NOT EXISTS store_staff_invites_open_unique
    ON store_staff_invites (store_id, email)
    WHERE accepted_at IS NULL;

CREATE INDEX IF NOT EXISTS store_staff_invites_store_idx
    ON store_staff_invites (store_id);
