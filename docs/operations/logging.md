# Logging (Best Practices)

## Goals
- Structured logs for API + worker processes.
- Request-level correlation with `x-request-id`.
- Minimal overhead; log formatting controlled by env.

## Format
- `LOG_FORMAT=json` for structured JSON logs (prod).
- Default: human-readable text (local dev).

## Correlation
- `x-request-id` is injected and logged in the HTTP trace span.
- `trace_id` is logged when OpenTelemetry is enabled; otherwise `request_id` is used as a fallback.
- Pass `x-request-id` across services when available.

## Health
- `/health` performs a DB ping; log failures as errors.

## Recommended Levels
- `RUST_LOG=info` (prod baseline)
- Use `debug` for local development.

## Notes
- The API server uses `tower_http::TraceLayer` for request spans.
- Workers should emit batch/loop metrics (done/failed) per cycle.
- Workers support `LOG_FORMAT=json` and `RUST_LOG` like the API.
- Responses include `x-request-id` when provided or generated.

## OpenTelemetry (Jaeger)
- Use OTLP exporter with Jaeger Collector.
- Enable with:
  - `ENABLE_OTEL=true`
  - `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317`
- Service names:
  - API: `rs-ecommerce`
  - Worker: `inventory-worker`
