-- Sample seed for tenant + store settings (adjust values as needed)
INSERT INTO tenants (id, name, type, default_currency, status, settings)
VALUES ('00000000-0000-0000-0000-000000000001', 'Sample Store', 'single_brand', 'JPY', 'active', '{}'::jsonb);

INSERT INTO vendors (id, tenant_id, name, commission_rate, status)
VALUES ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000001', 'Sample Store', 0, 'active');

INSERT INTO store_settings (
    tenant_id, store_name, legal_name, contact_email, contact_phone,
    address_prefecture, address_city, address_line1, address_line2, legal_notice,
    default_language, primary_domain, subdomain, https_enabled, currency,
    tax_mode, tax_rounding, order_initial_status, cod_enabled,
    cod_fee_amount, cod_fee_currency, bank_name, bank_branch, bank_account_type,
    bank_account_number, bank_account_name, theme, brand_color, logo_url, favicon_url
) VALUES (
    '00000000-0000-0000-0000-000000000001', 'Sample Store', 'Sample Co., Ltd.',
    'support@example.com', '03-0000-0000',
    'Tokyo', 'Shibuya', '1-2-3', NULL, '特商法表記',
    'ja', NULL, 'sample', true, 'JPY',
    'inclusive', 'round', 'pending_payment', true,
    330, 'JPY', 'Sample Bank', 'Shibuya', 'normal',
    '1234567', 'SAMPLE', 'default', '#000000', NULL, NULL
);

INSERT INTO mall_settings (tenant_id, enabled, commission_rate, vendor_approval_required)
VALUES ('00000000-0000-0000-0000-000000000001', false, 0, true);
