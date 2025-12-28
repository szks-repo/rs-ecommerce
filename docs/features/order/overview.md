# Feature: Order

## Purpose
- Manage order lifecycle, status transitions, and shipments.

## Scope
- Included:
  - Order list/read (backoffice)
  - Status updates (bank transfer / COD)
  - Shipment create/update
- Excluded:
  - Payment capture automation (future)
  - Returns/refunds (future)

## Domain Model (draft)
- Entities:
  - Order
  - Shipment
- Invariants:
  - Shipment belongs to order

## APIs
- BackofficeService.ListOrders
- BackofficeService.UpdateOrderStatus
- BackofficeService.CreateShipment / UpdateShipmentStatus
- StorefrontService.Checkout / GetOrder

## Data Model
- Tables:
  - orders
  - shipments

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
