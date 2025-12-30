-- Role & permission system for store staff.

CREATE TABLE IF NOT EXISTS store_roles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    key text NOT NULL,
    name text NOT NULL,
    description text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS store_roles_key_unique
    ON store_roles (store_id, key);

CREATE TABLE IF NOT EXISTS permissions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    key text NOT NULL,
    name text NOT NULL,
    description text
);

CREATE UNIQUE INDEX IF NOT EXISTS permissions_key_unique ON permissions (key);

CREATE TABLE IF NOT EXISTS store_role_permissions (
    role_id uuid NOT NULL REFERENCES store_roles(id) ON DELETE CASCADE,
    permission_id uuid NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS store_staff_roles (
    staff_id uuid NOT NULL REFERENCES store_staff(id) ON DELETE CASCADE,
    role_id uuid NOT NULL REFERENCES store_roles(id) ON DELETE CASCADE,
    PRIMARY KEY (staff_id, role_id)
);

-- Seed common permissions (idempotent).
INSERT INTO permissions (key, name, description) VALUES
    ('catalog.read', 'Catalog Read', 'Read products and variants'),
    ('catalog.write', 'Catalog Write', 'Create/update products and variants'),
    ('orders.read', 'Orders Read', 'Read orders'),
    ('orders.write', 'Orders Write', 'Update orders and shipments'),
    ('promotions.read', 'Promotions Read', 'Read promotions'),
    ('promotions.write', 'Promotions Write', 'Create/update promotions'),
    ('settings.read', 'Settings Read', 'Read store settings'),
    ('settings.write', 'Settings Write', 'Update store settings'),
    ('staff.manage', 'Staff Manage', 'Create/update staff and roles'),
    ('customers.read', 'Customers Read', 'Read customer profiles and identities'),
    ('customers.write', 'Customers Write', 'Create/update customer profiles and identities')
ON CONFLICT (key) DO NOTHING;
