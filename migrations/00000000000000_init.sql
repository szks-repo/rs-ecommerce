-- Consolidated schema for rs-ecommerce.
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS tenants (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    type text NOT NULL,
    default_currency text NOT NULL,
    status text NOT NULL,
    settings jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS vendors (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    name text NOT NULL,
    commission_rate numeric,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS stores (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    code text,
    name text NOT NULL,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS stores_tenant_idx ON stores (tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS stores_code_unique
    ON stores (code)
    WHERE code IS NOT NULL;

CREATE TABLE IF NOT EXISTS permissions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    key text NOT NULL,
    name text NOT NULL,
    description text
);

CREATE UNIQUE INDEX IF NOT EXISTS permissions_key_unique ON permissions (key);

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
    ('customers.write', 'Customers Write', 'Create/update customer profiles and identities'),
    ('audit.read', 'Audit Read', 'View audit logs'),
    ('auction.read', 'Auction Read', 'Read auctions and bids'),
    ('auction.write', 'Auction Write', 'Create/update auctions and place bids')
ON CONFLICT (key) DO NOTHING;

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

CREATE TABLE IF NOT EXISTS store_staff (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    email text,
    login_id text,
    phone text,
    password_hash text,
    role_id uuid REFERENCES store_roles(id),
    display_name text,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_staff_store_idx ON store_staff (store_id);
CREATE UNIQUE INDEX IF NOT EXISTS store_staff_email_unique
    ON store_staff (store_id, email) WHERE email IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS store_staff_login_id_unique
    ON store_staff (store_id, login_id) WHERE login_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS store_staff_phone_unique
    ON store_staff (store_id, phone) WHERE phone IS NOT NULL;

CREATE TABLE IF NOT EXISTS store_role_permissions (
    role_id uuid NOT NULL REFERENCES store_roles(id) ON DELETE CASCADE,
    permission_id uuid NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

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

CREATE TABLE IF NOT EXISTS customers (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    email text,
    name text,
    phone text,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS customers_tenant_email_idx ON customers (tenant_id, email);

CREATE TABLE IF NOT EXISTS store_profile_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_name text NOT NULL,
    legal_name text NOT NULL,
    contact_email text NOT NULL,
    contact_phone text NOT NULL,
    address_prefecture text NOT NULL,
    address_city text NOT NULL,
    address_line1 text NOT NULL,
    address_line2 text,
    legal_notice text NOT NULL,
    default_language text NOT NULL,
    primary_domain text,
    subdomain text,
    https_enabled bool NOT NULL DEFAULT true,
    currency text NOT NULL,
    order_initial_status text NOT NULL,
    time_zone text NOT NULL DEFAULT 'Asia/Tokyo',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_tax_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    tax_mode text NOT NULL,
    tax_rounding text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_payment_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    cod_enabled bool NOT NULL DEFAULT true,
    cod_fee_amount bigint NOT NULL DEFAULT 0,
    cod_fee_currency text NOT NULL DEFAULT 'JPY',
    bank_transfer_enabled bool NOT NULL DEFAULT true,
    bank_name text NOT NULL,
    bank_branch text NOT NULL,
    bank_account_type text NOT NULL,
    bank_account_number text NOT NULL,
    bank_account_name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_catalog_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    sku_code_regex text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_appearance_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    theme text NOT NULL,
    brand_color text NOT NULL,
    logo_url text,
    favicon_url text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_profile_settings_tenant_idx ON store_profile_settings (tenant_id);
CREATE INDEX IF NOT EXISTS store_tax_settings_tenant_idx ON store_tax_settings (tenant_id);
CREATE INDEX IF NOT EXISTS store_payment_settings_tenant_idx ON store_payment_settings (tenant_id);
CREATE INDEX IF NOT EXISTS store_catalog_settings_tenant_idx ON store_catalog_settings (tenant_id);
CREATE INDEX IF NOT EXISTS store_appearance_settings_tenant_idx ON store_appearance_settings (tenant_id);

CREATE TABLE IF NOT EXISTS shipping_zones (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    name text NOT NULL,
    domestic_only bool NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS shipping_zone_prefectures (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id uuid NOT NULL REFERENCES shipping_zones(id),
    prefecture_code text NOT NULL,
    prefecture_name text NOT NULL
);

CREATE TABLE IF NOT EXISTS shipping_rates (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id uuid NOT NULL REFERENCES shipping_zones(id),
    name text NOT NULL,
    min_subtotal_amount bigint,
    max_subtotal_amount bigint,
    fee_amount bigint NOT NULL,
    fee_currency text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS tax_rules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    name text NOT NULL,
    rate numeric NOT NULL,
    applies_to text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS mall_settings (
    tenant_id uuid PRIMARY KEY REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    enabled bool NOT NULL DEFAULT false,
    commission_rate numeric NOT NULL DEFAULT 0,
    vendor_approval_required bool NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS shipping_zones_tenant_idx ON shipping_zones (tenant_id);
CREATE INDEX IF NOT EXISTS shipping_zones_store_idx ON shipping_zones (store_id);
CREATE INDEX IF NOT EXISTS tax_rules_tenant_idx ON tax_rules (tenant_id);
CREATE INDEX IF NOT EXISTS tax_rules_store_idx ON tax_rules (store_id);
CREATE INDEX IF NOT EXISTS mall_settings_tenant_idx ON mall_settings (tenant_id);
CREATE INDEX IF NOT EXISTS mall_settings_store_idx ON mall_settings (store_id);

CREATE TABLE IF NOT EXISTS store_locations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    code text NOT NULL,
    name text NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (store_id, code)
);

CREATE TABLE IF NOT EXISTS products (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    vendor_id uuid REFERENCES vendors(id),
    title text NOT NULL,
    description text NOT NULL,
    status text NOT NULL,
    tax_rule_id uuid REFERENCES tax_rules(id),
    sale_start_at timestamptz,
    sale_end_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS products_store_status_idx
    ON products (store_id, status);
CREATE INDEX IF NOT EXISTS products_tax_rule_id_idx
    ON products (tax_rule_id);
CREATE INDEX IF NOT EXISTS products_sale_period_idx
    ON products (sale_start_at, sale_end_at);

CREATE TABLE IF NOT EXISTS product_categories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    parent_id uuid REFERENCES product_categories(id),
    name text NOT NULL,
    slug text NOT NULL,
    description text,
    status text NOT NULL DEFAULT 'active',
    position int NOT NULL DEFAULT 0,
    visibility_json jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (store_id, slug)
);

CREATE INDEX IF NOT EXISTS product_categories_store_parent_pos_idx
    ON product_categories (store_id, parent_id, position);
CREATE INDEX IF NOT EXISTS product_categories_store_status_idx
    ON product_categories (store_id, status);

CREATE TABLE IF NOT EXISTS product_category_closure (
    store_id uuid NOT NULL REFERENCES stores(id),
    ancestor_id uuid NOT NULL REFERENCES product_categories(id) ON DELETE CASCADE,
    descendant_id uuid NOT NULL REFERENCES product_categories(id) ON DELETE CASCADE,
    depth int NOT NULL,
    PRIMARY KEY (ancestor_id, descendant_id)
);

CREATE INDEX IF NOT EXISTS product_category_closure_store_ancestor_idx
    ON product_category_closure (store_id, ancestor_id);
CREATE INDEX IF NOT EXISTS product_category_closure_store_descendant_idx
    ON product_category_closure (store_id, descendant_id);

CREATE TABLE IF NOT EXISTS product_locations (
    product_id uuid NOT NULL REFERENCES products(id),
    location_id uuid NOT NULL REFERENCES store_locations(id),
    PRIMARY KEY (product_id, location_id)
);

CREATE TABLE IF NOT EXISTS store_digital_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    default_url_ttl_seconds int NOT NULL DEFAULT 86400,
    default_max_downloads int,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS product_digital_settings (
    product_id uuid PRIMARY KEY REFERENCES products(id),
    url_ttl_seconds int,
    max_downloads int,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS product_skus (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id uuid NOT NULL REFERENCES products(id),
    sku text NOT NULL,
    jan_code text,
    fulfillment_type text NOT NULL DEFAULT 'physical',
    price_amount bigint NOT NULL,
    price_currency text NOT NULL,
    compare_at_amount bigint,
    compare_at_currency text,
    status text NOT NULL,
    tax_rule_id uuid REFERENCES tax_rules(id),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS product_skus_product_sku_idx
    ON product_skus (product_id, sku);
CREATE INDEX IF NOT EXISTS product_skus_tax_rule_id_idx
    ON product_skus (tax_rule_id);

CREATE TABLE IF NOT EXISTS product_category_links (
    product_id uuid NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    category_id uuid NOT NULL REFERENCES product_categories(id) ON DELETE CASCADE,
    is_primary bool NOT NULL DEFAULT false,
    position int NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (product_id, category_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS product_category_links_primary_idx
    ON product_category_links (product_id)
    WHERE is_primary = true;
CREATE INDEX IF NOT EXISTS product_category_links_category_idx
    ON product_category_links (category_id, position);

CREATE TABLE IF NOT EXISTS product_variant_axes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id uuid NOT NULL REFERENCES products(id),
    name text NOT NULL,
    position int NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (product_id, name)
);

CREATE INDEX IF NOT EXISTS product_variant_axes_product_idx
    ON product_variant_axes (product_id, position);

CREATE TABLE IF NOT EXISTS variant_axis_values (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    variant_id uuid NOT NULL REFERENCES product_skus(id) ON DELETE CASCADE,
    axis_id uuid NOT NULL REFERENCES product_variant_axes(id) ON DELETE CASCADE,
    value text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (variant_id, axis_id)
);

CREATE INDEX IF NOT EXISTS variant_axis_values_variant_idx
    ON variant_axis_values (variant_id);

CREATE TABLE IF NOT EXISTS inventory_stocks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    location_id uuid NOT NULL REFERENCES store_locations(id),
    variant_id uuid NOT NULL REFERENCES product_skus(id),
    stock int NOT NULL,
    reserved int NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (variant_id, location_id)
);

CREATE TABLE IF NOT EXISTS store_media_assets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    provider text NOT NULL DEFAULT '',
    bucket text NOT NULL DEFAULT '',
    object_key text NOT NULL DEFAULT '',
    public_url text NOT NULL,
    content_type text,
    size_bytes bigint,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_media_assets_store_idx
    ON store_media_assets (store_id, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS store_media_assets_store_key_idx
    ON store_media_assets (store_id, object_key)
    WHERE object_key <> '';

CREATE TABLE IF NOT EXISTS sku_images (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    sku_id uuid NOT NULL REFERENCES product_skus(id),
    asset_id uuid NOT NULL REFERENCES store_media_assets(id),
    position int NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (sku_id, asset_id),
    UNIQUE (sku_id, position)
);

CREATE INDEX IF NOT EXISTS sku_images_sku_idx ON sku_images (sku_id);

CREATE TABLE IF NOT EXISTS store_digital_assets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    sku_id uuid NOT NULL REFERENCES product_skus(id),
    provider text NOT NULL,
    bucket text NOT NULL,
    object_key text NOT NULL,
    content_type text,
    size_bytes bigint,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS store_digital_assets_store_sku_idx
    ON store_digital_assets (store_id, sku_id, created_at);

CREATE TABLE IF NOT EXISTS carts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    customer_id uuid REFERENCES customers(id),
    status text NOT NULL DEFAULT 'active',
    expires_at timestamptz NOT NULL DEFAULT now() + interval '30 days',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS cart_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id uuid NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    sku_id uuid NOT NULL REFERENCES product_skus(id),
    location_id uuid REFERENCES store_locations(id),
    unit_price_amount bigint NOT NULL,
    unit_price_currency text NOT NULL,
    quantity int NOT NULL,
    fulfillment_type text NOT NULL,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS cart_items_cart_idx ON cart_items (cart_id);
CREATE INDEX IF NOT EXISTS cart_items_sku_idx ON cart_items (sku_id);

CREATE TABLE IF NOT EXISTS inventory_reservations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    cart_id uuid NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    cart_item_id uuid REFERENCES cart_items(id) ON DELETE CASCADE,
    sku_id uuid NOT NULL REFERENCES product_skus(id),
    location_id uuid REFERENCES store_locations(id),
    quantity int NOT NULL,
    status text NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (quantity > 0)
);

CREATE INDEX IF NOT EXISTS inventory_reservations_expiry_idx
    ON inventory_reservations (expires_at)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS inventory_reservations_cart_idx
    ON inventory_reservations (cart_id, status);

CREATE INDEX IF NOT EXISTS inventory_reservations_sku_idx
    ON inventory_reservations (sku_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS inventory_reservations_cart_item_active_idx
    ON inventory_reservations (cart_item_id)
    WHERE status = 'active' AND cart_item_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS inventory_reservation_requests (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    cart_id uuid NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    cart_item_id uuid REFERENCES cart_items(id) ON DELETE CASCADE,
    sku_id uuid NOT NULL REFERENCES product_skus(id),
    location_id uuid REFERENCES store_locations(id),
    quantity int NOT NULL,
    status text NOT NULL,
    is_hot boolean NOT NULL DEFAULT false,
    idempotency_key text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (quantity > 0)
);

CREATE INDEX IF NOT EXISTS inventory_reservation_requests_status_idx
    ON inventory_reservation_requests (status, is_hot, created_at);

CREATE INDEX IF NOT EXISTS inventory_reservation_requests_store_idx
    ON inventory_reservation_requests (store_id, status);

CREATE UNIQUE INDEX IF NOT EXISTS inventory_reservation_requests_idem_idx
    ON inventory_reservation_requests (idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS orders (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    customer_id uuid REFERENCES customers(id),
    status text NOT NULL,
    total_amount bigint NOT NULL,
    currency text NOT NULL,
    payment_method text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS orders_tenant_created_idx ON orders (tenant_id, created_at);

CREATE TABLE IF NOT EXISTS order_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id uuid NOT NULL REFERENCES orders(id),
    vendor_id uuid REFERENCES vendors(id),
    variant_id uuid NOT NULL REFERENCES product_skus(id),
    price_amount bigint NOT NULL,
    price_currency text NOT NULL,
    quantity int NOT NULL
);

CREATE TABLE IF NOT EXISTS shipments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id uuid NOT NULL REFERENCES orders(id),
    vendor_id uuid REFERENCES vendors(id),
    status text NOT NULL,
    tracking_no text,
    carrier text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS promotions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    code text NOT NULL,
    discount_type text NOT NULL,
    value_amount bigint NOT NULL,
    value_currency text NOT NULL,
    status text NOT NULL,
    starts_at timestamptz,
    ends_at timestamptz
);

CREATE TABLE IF NOT EXISTS digital_deliveries (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    order_item_id uuid NOT NULL REFERENCES order_items(id),
    provider text NOT NULL,
    object_key text NOT NULL,
    url_expires_at timestamptz,
    max_downloads int,
    downloaded_count int NOT NULL DEFAULT 0,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS customer_profiles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id uuid NOT NULL REFERENCES customers(id),
    store_id uuid NOT NULL REFERENCES stores(id),
    status text NOT NULL DEFAULT 'active',
    preferences jsonb NOT NULL DEFAULT '{}'::jsonb,
    name text NOT NULL DEFAULT '',
    email text,
    phone text,
    notes text,
    country_code text NOT NULL DEFAULT 'JP',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (customer_id, store_id)
);

CREATE TABLE IF NOT EXISTS customer_identities (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id uuid NOT NULL REFERENCES customers(id),
    tenant_id uuid REFERENCES tenants(id),
    identity_type text NOT NULL,
    identity_value text NOT NULL,
    verified boolean NOT NULL DEFAULT false,
    source text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, identity_type, identity_value)
);

CREATE TABLE IF NOT EXISTS customer_addresses (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id uuid NOT NULL REFERENCES customers(id),
    type text NOT NULL,
    name text NOT NULL,
    postal_code text NOT NULL,
    prefecture text NOT NULL,
    city text NOT NULL,
    line1 text NOT NULL,
    line2 text,
    phone text,
    country_code text NOT NULL DEFAULT 'JP',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS customer_profiles_store_idx ON customer_profiles (store_id);
CREATE INDEX IF NOT EXISTS customer_profiles_customer_idx ON customer_profiles (customer_id);
CREATE INDEX IF NOT EXISTS customer_identities_tenant_idx
    ON customer_identities (tenant_id);
CREATE INDEX IF NOT EXISTS customer_addresses_customer_idx ON customer_addresses (customer_id);

CREATE TABLE IF NOT EXISTS metafield_definitions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_type text NOT NULL,
    namespace text NOT NULL,
    key text NOT NULL,
    name text NOT NULL,
    description text,
    value_type text NOT NULL,
    is_list bool NOT NULL DEFAULT false,
    validations_json jsonb NOT NULL DEFAULT '{}'::jsonb,
    visibility_json jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (owner_type, namespace, key)
);

CREATE INDEX IF NOT EXISTS metafield_definitions_owner_idx
    ON metafield_definitions (owner_type, namespace, key);

CREATE TABLE IF NOT EXISTS metafield_values (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id uuid NOT NULL REFERENCES metafield_definitions(id) ON DELETE CASCADE,
    owner_id uuid NOT NULL,
    value_json jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (definition_id, owner_id)
);

CREATE INDEX IF NOT EXISTS metafield_values_owner_idx
    ON metafield_values (owner_id);

CREATE TABLE IF NOT EXISTS store_db_routing (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    db_key text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS db_connections (
    db_key text PRIMARY KEY,
    kind text NOT NULL,
    host text NOT NULL,
    port int NOT NULL,
    database_name text NOT NULL,
    username text NOT NULL,
    password_secret_ref text NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS outbox_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid REFERENCES stores(id),
    aggregate_type text NOT NULL,
    aggregate_id text NOT NULL,
    event_type text NOT NULL,
    payload_json jsonb NOT NULL,
    status text NOT NULL DEFAULT 'pending',
    idempotency_key text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);

CREATE UNIQUE INDEX IF NOT EXISTS outbox_idempotency_unique
    ON outbox_events (tenant_id, idempotency_key);
CREATE INDEX IF NOT EXISTS outbox_pending_idx
    ON outbox_events (status, created_at);

CREATE TABLE IF NOT EXISTS processed_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    store_id uuid REFERENCES stores(id),
    event_id uuid NOT NULL REFERENCES outbox_events(id),
    processed_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, event_id, store_id)
);

CREATE TABLE IF NOT EXISTS store_sync_settings (
    store_id uuid PRIMARY KEY REFERENCES stores(id),
    tenant_id uuid NOT NULL REFERENCES tenants(id),
    customer_sync_enabled bool NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS auctions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    product_id uuid REFERENCES products(id),
    sku_id uuid REFERENCES product_skus(id),
    title text NOT NULL DEFAULT '',
    description text NOT NULL DEFAULT '',
    auction_type text NOT NULL,
    status text NOT NULL,
    start_at timestamptz NOT NULL,
    end_at timestamptz NOT NULL,
    bid_increment_amount bigint NOT NULL,
    bid_increment_currency text NOT NULL,
    start_price_amount bigint NOT NULL,
    start_price_currency text NOT NULL,
    reserve_price_amount bigint,
    reserve_price_currency text,
    buyout_price_amount bigint,
    buyout_price_currency text,
    current_price_amount bigint,
    current_price_currency text,
    current_bid_id uuid,
    winning_bid_id uuid,
    winning_price_amount bigint,
    winning_price_currency text,
    approved_by uuid,
    approved_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS auctions_store_status_idx
    ON auctions (store_id, status, end_at);

CREATE TABLE IF NOT EXISTS auction_bids (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    auction_id uuid NOT NULL REFERENCES auctions(id) ON DELETE CASCADE,
    store_id uuid NOT NULL REFERENCES stores(id),
    customer_id uuid REFERENCES customers(id),
    amount bigint NOT NULL,
    currency text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS auction_bids_auction_idx
    ON auction_bids (auction_id, created_at);

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

CREATE TABLE IF NOT EXISTS auction_auto_bids (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    auction_id uuid NOT NULL REFERENCES auctions(id) ON DELETE CASCADE,
    store_id uuid NOT NULL REFERENCES stores(id),
    customer_id uuid NOT NULL REFERENCES customers(id),
    max_amount bigint NOT NULL,
    currency text NOT NULL,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS auction_auto_bids_auction_customer_idx
    ON auction_auto_bids (auction_id, customer_id);

CREATE INDEX IF NOT EXISTS auction_auto_bids_auction_idx
    ON auction_auto_bids (auction_id, status, max_amount DESC, created_at ASC);

CREATE TABLE IF NOT EXISTS audit_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid REFERENCES stores(id),
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

CREATE INDEX IF NOT EXISTS audit_logs_store_created_idx
    ON audit_logs (store_id, created_at);
CREATE INDEX IF NOT EXISTS audit_logs_store_target_idx
    ON audit_logs (store_id, target_type, target_id);
CREATE INDEX IF NOT EXISTS audit_logs_store_actor_idx
    ON audit_logs (store_id, actor_id);
