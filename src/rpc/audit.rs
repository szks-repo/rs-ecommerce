use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{
    AppState, audit,
    pb::pb,
    rpc::json::{ConnectError, parse_request, require_tenant_id},
    shared::audit_action::AuditAction,
};

pub async fn list_audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<pb::ListAuditLogsResponse>), (StatusCode, Json<ConnectError>)> {
    let req = parse_request::<pb::ListAuditLogsRequest>(&headers, body)?;
    let tenant_id = require_tenant_id(req.tenant.clone())?;
    let (logs, page) = audit::service::list_audit_logs(&state, tenant_id, req).await?;
    Ok((
        StatusCode::OK,
        Json(pb::ListAuditLogsResponse {
            logs,
            page: Some(page),
        }),
    ))
}

pub async fn list_audit_actions(
    State(_state): State<AppState>,
    _headers: HeaderMap,
    _body: Bytes,
) -> Result<(StatusCode, Json<pb::ListAuditActionsResponse>), (StatusCode, Json<ConnectError>)> {
    use crate::shared::audit_action::ALL_AUDIT_ACTIONS;

    let actions = ALL_AUDIT_ACTIONS
        .iter()
        .map(|action| pb::AuditActionItem {
            r#type: audit_action_type(action),
            key: action.as_str().to_string(),
            label: action.label().to_string(),
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(pb::ListAuditActionsResponse { actions }),
    ))
}

fn audit_action_type(action: &AuditAction) -> i32 {
    match action {
        AuditAction::ProductCreate => pb::AuditActionType::AuditActionProductCreate as i32,
        AuditAction::ProductUpdate => pb::AuditActionType::AuditActionProductUpdate as i32,
        AuditAction::VariantCreate => pb::AuditActionType::AuditActionVariantCreate as i32,
        AuditAction::VariantUpdate => pb::AuditActionType::AuditActionVariantUpdate as i32,
        AuditAction::InventorySet => pb::AuditActionType::AuditActionInventorySet as i32,
        AuditAction::StoreSettingsUpdate => {
            pb::AuditActionType::AuditActionStoreSettingsUpdate as i32
        }
        AuditAction::StoreSettingsInitialize => {
            pb::AuditActionType::AuditActionStoreSettingsInitialize as i32
        }
        AuditAction::MallSettingsInitialize => {
            pb::AuditActionType::AuditActionMallSettingsInitialize as i32
        }
        AuditAction::MallSettingsUpdate => {
            pb::AuditActionType::AuditActionMallSettingsUpdate as i32
        }
        AuditAction::StoreLocationUpsert => {
            pb::AuditActionType::AuditActionStoreLocationUpsert as i32
        }
        AuditAction::StoreLocationDelete => {
            pb::AuditActionType::AuditActionStoreLocationDelete as i32
        }
        AuditAction::ShippingZoneUpsert => {
            pb::AuditActionType::AuditActionShippingZoneUpsert as i32
        }
        AuditAction::ShippingZoneDelete => {
            pb::AuditActionType::AuditActionShippingZoneDelete as i32
        }
        AuditAction::ShippingRateUpsert => {
            pb::AuditActionType::AuditActionShippingRateUpsert as i32
        }
        AuditAction::ShippingRateDelete => {
            pb::AuditActionType::AuditActionShippingRateDelete as i32
        }
        AuditAction::TaxRuleUpsert => pb::AuditActionType::AuditActionTaxRuleUpsert as i32,
        AuditAction::TaxRuleDelete => pb::AuditActionType::AuditActionTaxRuleDelete as i32,
        AuditAction::PromotionCreate => pb::AuditActionType::AuditActionPromotionCreate as i32,
        AuditAction::PromotionUpdate => pb::AuditActionType::AuditActionPromotionUpdate as i32,
        AuditAction::OrderUpdateStatus => pb::AuditActionType::AuditActionOrderUpdateStatus as i32,
        AuditAction::ShipmentCreate => pb::AuditActionType::AuditActionShipmentCreate as i32,
        AuditAction::ShipmentUpdateStatus => {
            pb::AuditActionType::AuditActionShipmentUpdateStatus as i32
        }
        AuditAction::IdentitySignIn => pb::AuditActionType::AuditActionIdentitySignIn as i32,
        AuditAction::IdentitySignOut => pb::AuditActionType::AuditActionIdentitySignOut as i32,
        AuditAction::IdentityStaffCreate => {
            pb::AuditActionType::AuditActionIdentityStaffCreate as i32
        }
        AuditAction::IdentityStaffUpdate => {
            pb::AuditActionType::AuditActionIdentityStaffUpdate as i32
        }
        AuditAction::IdentityStaffInvite => {
            pb::AuditActionType::AuditActionIdentityStaffInvite as i32
        }
        AuditAction::IdentityRoleCreate => {
            pb::AuditActionType::AuditActionIdentityRoleCreate as i32
        }
        AuditAction::IdentityRoleAssign => {
            pb::AuditActionType::AuditActionIdentityRoleAssign as i32
        }
        AuditAction::IdentityRoleUpdate => {
            pb::AuditActionType::AuditActionIdentityRoleUpdate as i32
        }
        AuditAction::IdentityRoleDelete => {
            pb::AuditActionType::AuditActionIdentityRoleDelete as i32
        }
        AuditAction::IdentityOwnerTransfer => {
            pb::AuditActionType::AuditActionIdentityOwnerTransfer as i32
        }
        AuditAction::CustomerCreate => pb::AuditActionType::AuditActionCustomerCreate as i32,
        AuditAction::CustomerUpdate => pb::AuditActionType::AuditActionCustomerUpdate as i32,
        AuditAction::CustomerIdentityUpsert => {
            pb::AuditActionType::AuditActionCustomerIdentityUpsert as i32
        }
        AuditAction::CustomerAddressUpsert => {
            pb::AuditActionType::AuditActionCustomerAddressUpsert as i32
        }
    }
}
