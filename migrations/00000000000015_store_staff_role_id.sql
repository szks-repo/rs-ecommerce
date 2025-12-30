-- Move store_staff.role to role_id for single-role model.

ALTER TABLE store_staff ADD COLUMN IF NOT EXISTS role_id uuid;

-- Ensure roles exist for current staff role keys.
INSERT INTO store_roles (id, store_id, key, name, description)
SELECT gen_random_uuid(), ss.store_id, ss.role, initcap(replace(ss.role, '_', ' ')), 'Seeded from staff role'
FROM store_staff ss
WHERE ss.role IS NOT NULL
  AND ss.role <> ''
  AND NOT EXISTS (
      SELECT 1
      FROM store_roles sr
      WHERE sr.store_id = ss.store_id AND sr.key = ss.role
  );

UPDATE store_staff ss
SET role_id = sr.id
FROM store_roles sr
WHERE sr.store_id = ss.store_id
  AND sr.key = ss.role
  AND ss.role_id IS NULL;

ALTER TABLE store_staff DROP COLUMN IF EXISTS role;

DROP TABLE IF EXISTS store_staff_roles;

ALTER TABLE store_staff
    ADD CONSTRAINT store_staff_role_id_fk
    FOREIGN KEY (role_id) REFERENCES store_roles(id);
