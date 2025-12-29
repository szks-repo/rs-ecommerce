# Store Settings Scope (Draft)

Store settings cover **operational configuration** for a store.  
Domain data such as customers/orders/products are **not** part of settings.

## Included in Store Settings

### 1) Store Profile
- Store name (display)
- Legal name
- Contact email / phone
- Business address
- Legal notices (特商法表記)
- Default language

### 2) Domain & Routing
- Primary domain
- Subdomain
- HTTPS enablement flag

### 3) Currency / Tax / Rounding
- Default currency (JPY)
- Tax mode: inclusive / exclusive
- Tax rounding: floor / round / ceil
- Tax rate table (per category, future)
  - Validation: rate must be between 0 and 1

### 4) Shipping (Japan-focused)
- Domestic only flag
- Shipping zones (prefecture groups)
- Zone-based rates
- COD fee rules
- Free shipping thresholds (optional)
  - Validation: min/max subtotal must be consistent, fee must be >= 0

### 4.5) Inventory Locations
- Store locations (warehouses / fulfillment nodes)
- Location status and metadata

### 5) Payment (Initial)
- Bank transfer account details
- COD enable/disable + fee

### 6) Order Flow
- Default order status on creation
- Status transitions allowed (optional rules)
- Notification triggers (email)

### 7) Roles / Permissions
- Admin roles
- Staff roles
- Role permissions preset

### 8) Mall (MVP included)
- Mall enable flag
- Vendor onboarding rules
- Commission rate
- Vendor approval flow

### 9) Storefront Display
- Theme selection
- Brand colors
- Logo / favicon

## Explicitly Excluded
- Customers / Customer segments
- Orders / Shipments / Returns (actual data)
- Products / Inventory / Pricing (actual data)

## Notes
- Settings are linked by `store_id` (tenant_id is kept for migration compatibility).
- Japan-focused shipping means address fields: prefecture/city/line1/line2.
- Future: multi-currency, international shipping, multi-language product catalogs.
