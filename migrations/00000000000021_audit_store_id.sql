DROP INDEX IF EXISTS audit_logs_tenant_created_idx;
DROP INDEX IF EXISTS audit_logs_target_idx;
DROP INDEX IF EXISTS audit_logs_actor_idx;

ALTER TABLE audit_logs
    DROP COLUMN IF EXISTS tenant_id,
    ADD COLUMN IF NOT EXISTS store_id uuid REFERENCES stores(id);

CREATE INDEX IF NOT EXISTS audit_logs_store_created_idx
    ON audit_logs (store_id, created_at);
CREATE INDEX IF NOT EXISTS audit_logs_store_target_idx
    ON audit_logs (store_id, target_type, target_id);
CREATE INDEX IF NOT EXISTS audit_logs_store_actor_idx
    ON audit_logs (store_id, actor_id);
