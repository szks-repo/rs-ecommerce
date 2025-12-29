use axum::{Json, http::StatusCode};
use sqlx::Row;
use serde_json::Value;

use crate::{
    AppState,
    pb::pb,
    infrastructure::{db, audit},
    rpc::json::ConnectError,
    shared::{
        audit_action::{AuditAction, OrderAuditAction, ShipmentAuditAction},
        ids::{parse_uuid, nullable_uuid},
        status::{order_status_from_string, order_status_to_string, payment_method_from_string, shipment_status_to_string},
    },
};

pub async fn list_orders(
    state: &AppState,
    tenant_id: String,
    status_filter: i32,
) -> Result<Vec<pb::OrderAdmin>, (StatusCode, Json<ConnectError>)> {
    let status = order_status_to_string(status_filter);
    let rows = if let Some(status) = status {
        sqlx::query(
            r#"
            SELECT id::text as id, customer_id::text as customer_id,
                   status, total_amount, currency, payment_method, created_at
            FROM orders
            WHERE tenant_id = $1 AND status = $2
            ORDER BY created_at DESC
            LIMIT 50
            "#,
        )
        .bind(&tenant_id)
        .bind(status)
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?
    } else {
        sqlx::query(
            r#"
            SELECT id::text as id, customer_id::text as customer_id,
                   status, total_amount, currency, payment_method, created_at
            FROM orders
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT 50
            "#,
        )
        .bind(&tenant_id)
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?
    };

    Ok(rows
        .into_iter()
        .map(|row| pb::OrderAdmin {
            id: row.get::<String, _>("id"),
            customer_id: row.get::<Option<String>, _>("customer_id").unwrap_or_default(),
            status: order_status_from_string(row.get::<String, _>("status")),
            total: Some(pb::Money {
                amount: row.get::<i64, _>("total_amount"),
                currency: row.get::<String, _>("currency"),
            }),
            payment_method: payment_method_from_string(row.get::<String, _>("payment_method")),
            created_at: None,
        })
        .collect())
}

pub async fn update_order_status(
    state: &AppState,
    tenant_id: String,
    req: pb::UpdateOrderStatusRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::OrderAdmin, (StatusCode, Json<ConnectError>)> {
    let before_status = sqlx::query("SELECT status FROM orders WHERE id = $1 AND tenant_id = $2")
        .bind(parse_uuid(&req.order_id, "order_id")?)
        .bind(&tenant_id)
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?
        .map(|row| row.get::<String, _>("status"));
    let status = order_status_to_string(req.status).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "status is required".to_string(),
            }),
        )
    })?;
    sqlx::query(
        r#"
        UPDATE orders SET status = $1, updated_at = now()
        WHERE id = $2 AND tenant_id = $3
        "#,
    )
    .bind(status)
    .bind(parse_uuid(&req.order_id, "order_id")?)
    .bind(&tenant_id)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let order = pb::OrderAdmin {
        id: req.order_id,
        customer_id: String::new(),
        status: req.status,
        total: None,
        payment_method: pb::PaymentMethod::Unspecified as i32,
        created_at: None,
    };

    let before_json = before_status.map(|s| serde_json::json!({ "status": s }));
    let after_json = Some(serde_json::json!({ "status": status }));
    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            OrderAuditAction::UpdateStatus.into(),
            Some("order"),
            Some(order.id.clone()),
            before_json,
            after_json,
            _actor,
        ),
    )
    .await?;

    Ok(order)
}

pub async fn create_shipment(
    state: &AppState,
    req: pb::CreateShipmentRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::ShipmentAdmin, (StatusCode, Json<ConnectError>)> {
    let tenant_id = tenant_id_for_order(state, &req.order_id).await?;
    let shipment_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO shipments (id, order_id, vendor_id, status, tracking_no, carrier)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(shipment_id)
    .bind(parse_uuid(&req.order_id, "order_id")?)
    .bind(nullable_uuid(req.vendor_id.clone()))
    .bind(shipment_status_to_string(req.status))
    .bind(req.tracking_no.clone())
    .bind(req.carrier.clone())
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let shipment = pb::ShipmentAdmin {
        id: shipment_id.to_string(),
        order_id: req.order_id,
        vendor_id: req.vendor_id,
        status: req.status,
        tracking_no: req.tracking_no,
        carrier: req.carrier,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id,
            ShipmentAuditAction::Create.into(),
            Some("shipment"),
            Some(shipment.id.clone()),
            None,
            to_json_opt(Some(shipment.clone())),
            _actor,
        ),
    )
    .await?;

    Ok(shipment)
}

pub async fn update_shipment_status(
    state: &AppState,
    req: pb::UpdateShipmentStatusRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::ShipmentAdmin, (StatusCode, Json<ConnectError>)> {
    let tenant_id = tenant_id_for_shipment(state, &req.shipment_id).await?;
    let before_status = sqlx::query("SELECT status FROM shipments WHERE id = $1")
        .bind(parse_uuid(&req.shipment_id, "shipment_id")?)
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?
        .map(|row| row.get::<String, _>("status"));
    sqlx::query(
        r#"
        UPDATE shipments
        SET status = $1, tracking_no = $2, carrier = $3, updated_at = now()
        WHERE id = $4
        "#,
    )
    .bind(shipment_status_to_string(req.status))
    .bind(req.tracking_no.clone())
    .bind(req.carrier.clone())
    .bind(parse_uuid(&req.shipment_id, "shipment_id")?)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let shipment = pb::ShipmentAdmin {
        id: req.shipment_id,
        order_id: String::new(),
        vendor_id: String::new(),
        status: req.status,
        tracking_no: req.tracking_no,
        carrier: req.carrier,
    };

    let before_json = before_status.map(|s| serde_json::json!({ "status": s }));
    let after_json = Some(serde_json::json!({ "status": shipment_status_to_string(req.status) }));
    let _ = audit::record(
        state,
        audit_input(
            tenant_id,
            ShipmentAuditAction::UpdateStatus.into(),
            Some("shipment"),
            Some(shipment.id.clone()),
            before_json,
            after_json,
            _actor,
        ),
    )
    .await?;

    Ok(shipment)
}

fn audit_input(
    tenant_id: String,
    action: AuditAction,
    target_type: Option<&str>,
    target_id: Option<String>,
    before_json: Option<Value>,
    after_json: Option<Value>,
    actor: Option<pb::ActorContext>,
) -> audit::AuditInput {
    let (actor_id, actor_type) = actor_fields(actor);
    audit::AuditInput {
        tenant_id,
        actor_id,
        actor_type,
        action,
        target_type: target_type.map(|v| v.to_string()),
        target_id,
        request_id: None,
        ip_address: None,
        user_agent: None,
        before_json,
        after_json,
        metadata_json: None,
    }
}

fn actor_fields(actor: Option<pb::ActorContext>) -> (Option<String>, String) {
    let actor_id = actor
        .as_ref()
        .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
    let actor_type = actor
        .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type) })
        .unwrap_or_else(|| "system".to_string());
    (actor_id, actor_type)
}

fn to_json_opt<T: serde::Serialize>(value: Option<T>) -> Option<Value> {
    value.and_then(|v| serde_json::to_value(v).ok())
}

async fn tenant_id_for_order(
    state: &AppState,
    order_id: &str,
) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM orders WHERE id = $1")
        .bind(parse_uuid(order_id, "order_id")?)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
    Ok(row.get("tenant_id"))
}

async fn tenant_id_for_shipment(
    state: &AppState,
    shipment_id: &str,
) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT o.tenant_id::text as tenant_id
        FROM orders o
        JOIN shipments s ON s.order_id = o.id
        WHERE s.id = $1
        "#,
    )
    .bind(parse_uuid(shipment_id, "shipment_id")?)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    Ok(row.get("tenant_id"))
}
