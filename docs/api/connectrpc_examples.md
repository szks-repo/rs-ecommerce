# ConnectRPC JSON Examples (Draft)

These examples use JSON-only ConnectRPC framing (proto JSON mapping).
Requests are POST to `/rpc/{fully.qualified.Service/Method}` with `Content-Type: application/json`.
Actor can be provided either in the request body (`actor`) or via headers:
- `Authorization: Bearer <token>` (mapped to `actorId`, `actorType` from token claims)
- `x-actor-id`, `x-actor-type` (manual override for local/dev)
When `Authorization` is present, the server should treat it as authoritative and ignore the body `actor`.

## StorefrontService.ListProducts

Request:
```json
{
  "tenant": { "tenantId": "tenant_123" },
  "page": { "pageSize": 20 }
}
```

Response (empty list):
```json
{
  "products": [],
  "page": { "nextPageToken": "" }
}
```

## BackofficeService.UpdateOrderStatus

Request:
```json
{
  "tenant": { "tenantId": "tenant_123" },
  "orderId": "11111111-1111-1111-1111-111111111111",
  "status": "PENDING_SHIPMENT",
  "actor": { "actorId": "admin_123", "actorType": "admin" }
}
```

Response:
```json
{
  "order": {
    "id": "11111111-1111-1111-1111-111111111111",
    "status": "PENDING_SHIPMENT"
  }
}
```

## BackofficeService.CreatePromotion

Request:
```json
{
  "tenant": { "tenantId": "tenant_123" },
  "code": "WELCOME10",
  "discountType": "percent",
  "value": { "currency": "JPY", "amount": 10 },
  "status": "active",
  "startsAt": "2025-01-01T00:00:00Z",
  "endsAt": "2025-02-01T00:00:00Z",
  "actor": { "actorId": "admin_123", "actorType": "admin" }
}
```

Response:
```json
{
  "promotion": {
    "id": "generated",
    "code": "WELCOME10",
    "discountType": "percent",
    "value": { "currency": "JPY", "amount": 10 },
    "status": "active"
  }
}
```

## StoreSettingsService.GetStoreSettings

Request:
```json
{
  "tenant": { "tenantId": "tenant_123" }
}
```

Response:
```json
{
  "settings": {
    "storeName": "Example Store",
    "legalName": "Example Co., Ltd.",
    "contactEmail": "support@example.com",
    "contactPhone": "03-0000-0000",
    "addressPrefecture": "Tokyo",
    "addressCity": "Shibuya",
    "addressLine1": "1-2-3",
    "legalNotice": "特商法表記...",
    "defaultLanguage": "ja",
    "currency": "JPY",
    "taxMode": "inclusive",
    "taxRounding": "round",
    "orderInitialStatus": "PENDING_PAYMENT",
    "codEnabled": true,
    "codFee": { "currency": "JPY", "amount": 330 }
  }
}
```

## StoreSettingsService.UpdateStoreSettings

Request:
```json
{
  "tenant": { "tenantId": "tenant_123" },
  "settings": {
    "storeName": "Example Store",
    "legalName": "Example Co., Ltd.",
    "contactEmail": "support@example.com",
    "contactPhone": "03-0000-0000",
    "addressPrefecture": "Tokyo",
    "addressCity": "Shibuya",
    "addressLine1": "1-2-3",
    "legalNotice": "特商法表記...",
    "defaultLanguage": "ja",
    "currency": "JPY",
    "taxMode": "inclusive",
    "taxRounding": "round",
    "orderInitialStatus": "PENDING_PAYMENT",
    "codEnabled": true,
    "codFee": { "currency": "JPY", "amount": 330 },
    "bankName": "Example Bank",
    "bankBranch": "Shibuya",
    "bankAccountType": "normal",
    "bankAccountNumber": "1234567",
    "bankAccountName": "EXAMPLE"
  },
  "actor": { "actorId": "admin_123", "actorType": "admin" }
}
```

Response:
```json
{
  "settings": {
    "storeName": "Example Store",
    "legalName": "Example Co., Ltd."
  }
}
```

## SetupService.InitializeStore

Request:
```json
{
  "storeName": "Example Store",
  "ownerEmail": "owner@example.com",
  "ownerPassword": "your-password",
  "actor": { "actorId": "admin_123", "actorType": "admin" }
}
```

Response:
```json
{
  "tenantId": "generated",
  "storeId": "generated",
  "ownerStaffId": "generated",
  "vendorId": "generated"
}
```

## cURL Example (InitializeStore)

```bash
curl -X POST http://localhost:8080/rpc/ecommerce.v1.SetupService/InitializeStore \
  -H 'Content-Type: application/json' \
  -d '{
    "storeName": "Example Store",
    "ownerEmail": "owner@example.com",
    "ownerPassword": "your-password",
    "actor": { "actorId": "admin_123", "actorType": "admin" }
  }'
```

## BackofficeService.CreateProduct

Request:
```json
{
  "tenant": { "tenantId": "tenant_123" },
  "vendorId": "vendor_001",
  "title": "Sample Product",
  "description": "Example description",
  "status": "active",
  "actor": { "actorId": "admin_123", "actorType": "admin" }
}
```

Response:
```json
{
  "product": {
    "id": "generated",
    "vendorId": "vendor_001",
    "title": "Sample Product",
    "description": "Example description",
    "status": "active"
  }
}
```

## AuditService.ListAuditLogs

Request:
```json
{
  "tenant": { "tenantId": "tenant_123" },
  "action": "store_settings.update",
  "actorId": "admin_123",
  "fromTime": "2025-01-01T00:00:00Z",
  "toTime": "2025-01-31T23:59:59Z",
  "page": { "pageSize": 50, "pageToken": "0" }
}
```

Response:
```json
{
  "logs": [
    {
      "id": "generated",
      "actorId": "admin_123",
      "actorType": "admin",
      "action": "store_settings.update",
      "targetType": "store_settings",
      "targetId": "tenant_123",
      "createdAt": "2025-01-10T12:34:56Z"
    }
  ],
  "page": { "nextPageToken": "50" }
}
```
