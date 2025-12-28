CREATE TABLE IF NOT EXISTS audit_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid REFERENCES tenants(id),
    actor_id text,
    actor_type text NOT NULL,
    action text NOT NULL,
    target_type text,
    target_id text,
    request_id text,
    ip_address text,
    user_agent text,
    before_json jsonb,
    after_json jsonb,
    metadata_json jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS audit_logs_tenant_created_idx ON audit_logs (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS audit_logs_target_idx ON audit_logs (tenant_id, target_type, target_id);
CREATE INDEX IF NOT EXISTS audit_logs_actor_idx ON audit_logs (tenant_id, actor_id);
