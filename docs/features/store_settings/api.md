# Store Settings API (Draft)

ConnectRPC JSON-only endpoints under `/rpc/ecommerce.v1.StoreSettingsService/*`.

## Store Settings
- GetStoreSettings
  - input: tenant
  - output: settings
- UpdateStoreSettings
  - input: tenant, settings
  - output: settings
- InitializeStoreSettings
  - input: tenant, settings, mall
  - output: settings, mall

## Mall Settings
- GetMallSettings
  - input: tenant
  - output: mall
- UpdateMallSettings
  - input: tenant, mall
  - output: mall

## Shipping Zones
- ListShippingZones
  - input: tenant
  - output: zones[]
- UpsertShippingZone
  - input: tenant, zone
  - output: zone
- DeleteShippingZone
  - input: tenant, zone_id
  - output: deleted

## Shipping Rates
- ListShippingRates
  - input: tenant, zone_id
  - output: rates[]
- UpsertShippingRate
  - input: tenant, rate
  - output: rate
- DeleteShippingRate
  - input: tenant, rate_id
  - output: deleted

## Tax Rules
- ListTaxRules
  - input: tenant
  - output: rules[]
- UpsertTaxRule
  - input: tenant, rule
  - output: rule
- DeleteTaxRule
  - input: tenant, rule_id
  - output: deleted

## Setup
- InitializeStore (SetupService)
  - input: tenant_name, settings, mall, default_zone, default_rate, default_tax_rule
  - output: tenant_id, vendor_id, settings, mall
