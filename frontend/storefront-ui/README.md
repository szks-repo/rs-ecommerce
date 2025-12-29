# Storefront UI

Minimal storefront preview for rs-ecommerce.

## Setup

```bash
npm install
```

## Run

```bash
# optional: set tenant id for storefront requests
export NEXT_PUBLIC_TENANT_ID=your-tenant-uuid

npm run dev
```

Then open http://localhost:3001.

## Notes

- Uses the StorefrontService ConnectRPC JSON endpoints.
- Product detail route: `/products/:productId`.
