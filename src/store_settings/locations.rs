use axum::{Json, http::StatusCode};

use crate::{
    AppState,
    pb::pb,
    infrastructure::audit,
    rpc::json::ConnectError,
    shared::{
        audit_action::StoreLocationAuditAction,
        ids::{parse_uuid, StoreId, TenantId},
    },
    store_settings::repository::{PgStoreSettingsRepository, StoreSettingsRepository},
};

use crate::shared::audit_helpers::{audit_input, to_json_opt};

pub async fn list_store_locations(
    state: &AppState,
    store_id: String,
) -> Result<Vec<pb::StoreLocation>, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let rows = repo.list_store_locations(&store_uuid.as_uuid()).await?;
    Ok(rows
        .into_iter()
        .map(|row| pb::StoreLocation {
            id: row.id,
            code: row.code,
            name: row.name,
            status: row.status,
        })
        .collect())
}

pub async fn upsert_store_location(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    location: pb::StoreLocation,
    actor: Option<pb::ActorContext>,
) -> Result<pb::StoreLocation, (StatusCode, Json<ConnectError>)> {
    validate_store_location(&location)?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let location_id = if location.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&location.id, "location_id")?
    };

    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(crate::infrastructure::db::error)?;
    if location.id.is_empty() {
        repo.insert_store_location_tx(
            &mut tx,
            &location_id,
            &tenant_uuid.as_uuid(),
            &store_uuid.as_uuid(),
            &location,
        )
        .await?;
    } else {
        repo.update_store_location_tx(
            &mut tx,
            &location_id,
            &store_uuid.as_uuid(),
            &location,
        )
        .await?;
    }

    let updated = pb::StoreLocation {
        id: location_id.to_string(),
        code: location.code,
        name: location.name,
        status: location.status,
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            tenant_id.clone(),
            StoreLocationAuditAction::Upsert.into(),
            Some("store_location"),
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

pub async fn delete_store_location(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    location_id: String,
    actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    let repo = PgStoreSettingsRepository::new(&state.db);
    let mut tx = state.db.begin().await.map_err(crate::infrastructure::db::error)?;
    let rows = repo
        .delete_store_location_tx(
            &mut tx,
            &parse_uuid(&location_id, "location_id")?,
            &store_uuid.as_uuid(),
        )
        .await?;
    let deleted = rows > 0;
    if deleted {
        audit::record_tx(
            &mut tx,
            audit_input(
                tenant_id.clone(),
                StoreLocationAuditAction::Delete.into(),
                Some("store_location"),
                Some(location_id),
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

pub fn validate_store_location(
    location: &pb::StoreLocation,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if location.code.is_empty() || location.name.is_empty() || location.status.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "location code/name/status are required".to_string(),
            }),
        ));
    }
    Ok(())
}
