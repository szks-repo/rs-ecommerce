CREATE TABLE IF NOT EXISTS store_staff_refresh_tokens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    staff_id uuid NOT NULL REFERENCES store_staff(id),
    session_id uuid NOT NULL REFERENCES store_staff_sessions(id),
    token_hash text NOT NULL,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    replaced_by uuid,
    last_used_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS store_staff_refresh_tokens_hash_key
    ON store_staff_refresh_tokens (token_hash);
CREATE INDEX IF NOT EXISTS store_staff_refresh_tokens_store_idx
    ON store_staff_refresh_tokens (store_id, revoked_at);
CREATE INDEX IF NOT EXISTS store_staff_refresh_tokens_staff_idx
    ON store_staff_refresh_tokens (staff_id, revoked_at);
CREATE INDEX IF NOT EXISTS store_staff_refresh_tokens_session_idx
    ON store_staff_refresh_tokens (session_id, revoked_at);
