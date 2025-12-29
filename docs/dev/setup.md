# Development Setup (Docker Compose)

## Prerequisites
- Docker Desktop (or Docker Engine + Compose)

## Services
- app (Rust)
- db (PostgreSQL 16)
- redis (Redis 7)
- meilisearch (Meilisearch 1.6)

## Environment
The compose file provides defaults:
- DATABASE_URL=postgres://rs:rs@db:5432/rs_ecommerce
- REDIS_URL=redis://redis:6379
- MEILI_URL=http://meilisearch:7700
- MEILI_MASTER_KEY=local-master-key

## Start
```bash
docker compose up --build
```

## Frontend (Admin UI)
```bash
cd frontend/admin-ui
npm install
npm run dev
```

## ConnectRPC Type Generation (Frontend)
```bash
# Install Buf CLI if not present (https://buf.build)
buf --version
buf generate
```

Generated files:
- `frontend/admin-ui/src/gen`

## Initialize Store (example)
```bash
curl -X POST http://localhost:8080/rpc/ecommerce.v1.SetupService/InitializeStore \
  -H 'Content-Type: application/json' \
  -d '{
    "tenantName": "Example Store",
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
      "bankAccountName": "EXAMPLE",
      "theme": "default",
      "brandColor": "#000000"
    }
  }'
```

## Stop
```bash
docker compose down
```

## Notes
- The current app only prints "Hello, world!" and does not bind port 8080 yet.
- Once an HTTP server is added, it should listen on 0.0.0.0:8080 for container access.
