# Store Settings API (Draft)

ConnectRPC JSON-only endpoints under `/rpc/ecommerce.v1.StoreSettingsService/*`.

## Store Settings
- GetStoreSettings
  - input: store (preferred), tenant (fallback)
  - output: settings
- UpdateStoreSettings
  - input: store (preferred), tenant (fallback), settings
  - output: settings
- InitializeStoreSettings
  - input: store (preferred), tenant (fallback), settings, mall
  - output: settings, mall

## Mall Settings
- GetMallSettings
  - input: store (preferred), tenant (fallback)
  - output: mall
- UpdateMallSettings
  - input: store (preferred), tenant (fallback), mall
  - output: mall

## Store Locations (Inventory)
- ListStoreLocations
  - input: store (preferred), tenant (fallback)
  - output: locations[]
- UpsertStoreLocation
  - input: store (preferred), tenant (fallback), location
  - output: location
- DeleteStoreLocation
  - input: store (preferred), tenant (fallback), location_id
  - output: deleted

## Shipping Zones
- ListShippingZones
  - input: store (preferred), tenant (fallback)
  - output: zones[]
- UpsertShippingZone
  - input: store (preferred), tenant (fallback), zone
  - output: zone
- DeleteShippingZone
  - input: store (preferred), tenant (fallback), zone_id
  - output: deleted

## Shipping Rates
- ListShippingRates
  - input: store (preferred), tenant (fallback), zone_id
  - output: rates[]
- UpsertShippingRate
  - input: store (preferred), tenant (fallback), rate
  - output: rate
- DeleteShippingRate
  - input: store (preferred), tenant (fallback), rate_id
  - output: deleted

## Tax Rules
- ListTaxRules
  - input: store (preferred), tenant (fallback)
  - output: rules[]
- UpsertTaxRule
  - input: store (preferred), tenant (fallback), rule
  - output: rule
- DeleteTaxRule
  - input: store (preferred), tenant (fallback), rule_id
  - output: deleted

## Setup
- InitializeStore (SetupService)
  - input: store_name, owner_email, owner_password, owner_login_id
  - output: tenant_id, store_id, owner_staff_id, vendor_id
