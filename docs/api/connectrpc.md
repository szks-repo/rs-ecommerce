# ConnectRPC Setup (Draft)

This project uses ConnectRPC as the API protocol. The proto definitions live in `proto/`.
The current approach is:
- Use axum as the HTTP layer
- Use prost for protobuf message types
- Use pbjson for JSON <-> protobuf mapping
- Start with JSON-only ConnectRPC framing (application/json)

## JSON Mapping Rules (pbjson)
- Request/response bodies follow proto JSON mapping (camelCase fields).
- Enums are serialized as their string names (e.g., `PENDING_PAYMENT`).
- `google.protobuf.Timestamp` is represented as RFC3339 string (e.g., `"2025-01-02T03:04:05Z"`).
- Unknown fields are rejected by JSON deserialization.

## Errors
- Errors are returned as JSON with HTTP status codes.
- Format:
```json
{
  "code": "invalid_argument",
  "message": "reason"
}
```
- Current codes: `invalid_argument`, `unsupported_media_type`, `internal`.

## Authentication (draft)
- Requests may include `Authorization: Bearer <token>`.
- The middleware currently maps tokens to `ActorContext` for audit logging.
- Full JWT verification design is documented in `docs/api/auth.md`.

## Endpoints
- POST `/rpc/{fully.qualified.Service/Method}`
- Example: `/rpc/ecommerce.v1.StorefrontService/ListProducts`

## Intended Flow
1) Generate Rust types from `proto/`
2) Implement ConnectRPC JSON request/response framing in axum handlers
3) Implement handlers for StorefrontService / BackofficeService
4) Mount ConnectRPC routes under `/rpc`

## Next Decisions
- Decide if we adopt a dedicated ConnectRPC Rust runtime (if available) or keep axum + custom framing
- Decide when to add binary protobuf support
