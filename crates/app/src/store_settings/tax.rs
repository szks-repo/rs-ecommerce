use axum::{Json, http::StatusCode};

use crate::{
    AppState,
    infrastructure::audit,
    pb::pb,
    rpc::json::ConnectError,
    shared::{
        audit_action::TaxRuleAuditAction,
        ids::{StoreId, TenantId, parse_uuid},
    },
    store_settings::repository::{PgStoreSettingsRepository, StoreSettingsRepository},
};

use crate::shared::audit_helpers::{audit_input, to_json_opt};

pub async fn list_tax_rules(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::TaxRule>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let rows = repo.list_tax_rules(&store_uuid.as_uuid()).await?;
    Ok(rows
        .into_iter()
        .map(|row| pb::TaxRule {
            id: row.id,
            name: row.name,
            rate: row.rate,
            applies_to: row.applies_to,
        })
        .collect())
}

pub async fn upsert_tax_rule(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    rule: pb::TaxRule,
    actor: Option<pb::ActorContext>,
) -> Result<pb::TaxRule, (StatusCode, Json<ConnectError>)> {
    validate_tax_rule(&rule)?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let rule_id = if rule.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&rule.id, "rule_id")?
    };

    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(crate::infrastructure::db::error)?;
    if rule.id.is_empty() {
        repo.insert_tax_rule_tx(&mut tx, &rule_id, &store_uuid.as_uuid(), &tenant_uuid.as_uuid(), &rule)
            .await?;
    } else {
        repo.update_tax_rule_tx(&mut tx, &rule_id, &store_uuid.as_uuid(), &rule)
            .await?;
    }

    let updated = pb::TaxRule {
        id: rule_id.to_string(),
        name: rule.name,
        rate: rule.rate,
        applies_to: rule.applies_to,
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            TaxRuleAuditAction::Upsert.into(),
            Some("tax_rule"),
            Some(updated.id.clone()),
            None,
            to_json_opt(Some(updated.clone())),
            actor.clone(),
        ),
    )
    .await?;

    tx.commit().await.map_err(crate::infrastructure::db::error)?;
    Ok(updated)
}

pub async fn delete_tax_rule(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    rule_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(crate::infrastructure::db::error)?;
    let rows = repo
        .delete_tax_rule_tx(&mut tx, &parse_uuid(&rule_id, "rule_id")?, &store_uuid.as_uuid())
        .await?;
    let deleted = rows > 0;
    if deleted {
        audit::record_tx(
            &mut tx,
            audit_input(
                Some(store_id.clone()),
                TaxRuleAuditAction::Delete.into(),
                Some("tax_rule"),
                Some(rule_id),
                None,
                None,
                actor.clone(),
            ),
        )
        .await?;
    }
    tx.commit().await.map_err(crate::infrastructure::db::error)?;
    Ok(deleted)
}

pub fn validate_tax_rule(rule: &pb::TaxRule) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if rule.name.is_empty() || rule.applies_to.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "tax rule name/applies_to are required".to_string(),
            }),
        ));
    }
    if !(0.0..=1.0).contains(&rule.rate) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "tax rule rate must be between 0 and 1".to_string(),
            }),
        ));
    }
    Ok(())
}
