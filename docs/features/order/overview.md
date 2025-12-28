# Feature: Order

## Purpose
- Manage order lifecycle, status transitions, and shipments.
- Support digital fulfillment with signed URLs.

## Scope
- Included:
  - Order list/read (backoffice)
  - Status updates (bank transfer / COD)
  - Shipment create/update
  - Issue digital delivery links for digital variants
- Excluded:
  - Payment capture automation (future)
  - Returns/refunds (future)

## Domain Model (draft)
- Entities:
  - Order
  - OrderItem
  - Shipment
  - DigitalDelivery
- Invariants:
  - Shipment belongs to order
- Digital deliveries exist only for digital variants
  - URL expiry / max downloads are resolved per product or store defaults

## APIs
- BackofficeService.ListOrders
- BackofficeService.UpdateOrderStatus
- BackofficeService.CreateShipment / UpdateShipmentStatus
- StorefrontService.Checkout / GetOrder

## Data Model
- Tables:
  - orders
  - order_items
  - shipments
  - digital_deliveries

## Flows
- Checkout:
  1. Create order
  2. Set initial status

## Audit
- Actions:
  - order.update_status
  - shipment.create
  - shipment.update_status

## Open Questions
- Status transition rules (formalize)
- Split shipments for multi-vendor
