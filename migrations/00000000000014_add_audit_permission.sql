-- Add audit permission key for role management.
INSERT INTO permissions (key, name, description)
VALUES ('audit.read', 'Audit Read', 'View audit logs')
ON CONFLICT (key) DO NOTHING;
