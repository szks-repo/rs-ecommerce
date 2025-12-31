CREATE TABLE IF NOT EXISTS store_staff_sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    staff_id uuid NOT NULL REFERENCES store_staff(id),
    ip_address text,
    user_agent text,
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    revoked_at timestamptz,
    expires_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_staff_sessions_store_idx
    ON store_staff_sessions (store_id, revoked_at);
CREATE INDEX IF NOT EXISTS store_staff_sessions_staff_idx
    ON store_staff_sessions (staff_id, revoked_at);
