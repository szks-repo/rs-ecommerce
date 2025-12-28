# Store Settings Form Spec (Draft)

## Required Fields
- Store name
- Legal name
- Contact email / phone
- Address (prefecture, city, line1)
- Legal notice (特商法表記)
- Default language
- Currency (JPY)
- Tax mode / rounding
- Order initial status
- COD enabled flag + COD fee (if enabled)
- Bank transfer account details
- Theme / brand color

## Optional Fields
- Address line2
- Primary domain / subdomain
- Logo / favicon

## Input Rules
- Tax rate: 0.0 - 1.0
- Shipping rate: fee >= 0
- Shipping rate range: min <= max
- Prefecture code: JP-01 .. JP-47
- Bank account number: numeric, length 7 (recommended)
- COD fee required when COD is enabled

## Defaults
- HTTPS enabled: true
- Language: ja
- Currency: JPY
- Tax mode: inclusive
- Tax rounding: round
- Order initial status: pending_payment

## Initial Setup
- Setup flow creates tenant + vendor + store settings + mall settings
- Optional: default shipping zone/rate and tax rule
- Validation: tenant_name must be unique
