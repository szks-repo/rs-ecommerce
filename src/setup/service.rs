use crate::{
    AppState,
    infrastructure::audit,
    infrastructure::db,
    pb::pb,
    rpc::json::ConnectError,
    shared::audit_action::{
        IdentityAuditAction, MallSettingsAuditAction, StoreSettingsAuditAction,
    },
    shared::validation::StoreCode,
    store_settings::{
        repository::{PgStoreSettingsRepository, StoreSettingsRepository},
        service::{default_mall_settings, default_store_settings},
    },
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use axum::{Json, http::StatusCode};
use rand_core::OsRng;

pub async fn initialize_store(
    state: &AppState,
    req: pb::InitializeStoreRequest,
) -> Result<pb::InitializeStoreResponse, (StatusCode, Json<ConnectError>)> {
    let _actor = req.actor.clone();
    if req.store_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "store_name is required".to_string(),
            }),
        ));
    }
    if req.owner_email.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "owner_email is required".to_string(),
            }),
        ));
    }
    if req.owner_password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "owner_password is required".to_string(),
            }),
        ));
    }
    let store_code = StoreCode::parse(&req.store_code)?;

    let mut tx = state.db.begin().await.map_err(db::error)?;

    let existing = sqlx::query("SELECT 1 FROM tenants WHERE name = $1 LIMIT 1")
        .bind(&req.store_name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::error)?;
    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::AlreadyExists,
                message: "store already exists".to_string(),
            }),
        ));
    }

    let existing_code = sqlx::query("SELECT 1 FROM stores WHERE code = $1 LIMIT 1")
        .bind(store_code.as_str())
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::error)?;
    if existing_code.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::AlreadyExists,
                message: "store_code already exists".to_string(),
            }),
        ));
    }

    let tenant_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO tenants (id, name, type, default_currency, status, settings)
        VALUES ($1, $2, $3, $4, $5, '{}'::jsonb)
        "#,
    )
    .bind(tenant_id)
    .bind(&req.store_name)
    .bind("single_brand")
    .bind("JPY")
    .bind("active")
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    let store_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO stores (id, tenant_id, name, status, code)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(store_id)
    .bind(tenant_id)
    .bind(&req.store_name)
    .bind("active")
    .bind(store_code.as_str())
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    let settings_repo = PgStoreSettingsRepository::new(&state.db);
    let store_settings = default_store_settings(req.store_name.clone());
    let payment = store_settings.payment.clone().unwrap_or_default();
    let (cod_fee_amount, cod_fee_currency) =
        crate::shared::money::money_to_parts(payment.cod_fee.clone())?;
    settings_repo
        .insert_store_settings_if_absent_tx(
            &mut tx,
            &tenant_id,
            &store_id,
            &store_settings,
            cod_fee_amount,
            cod_fee_currency,
        )
        .await?;

    let mall_settings = default_mall_settings();
    settings_repo
        .upsert_mall_settings_tx(&mut tx, &tenant_id, &store_id, &mall_settings)
        .await?;

    let owner_role_id = uuid::Uuid::new_v4();
    let staff_role_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO store_roles (id, store_id, key, name, description)
        VALUES
          ($1, $2, 'owner', 'Owner', 'Full access owner role'),
          ($3, $2, 'staff', 'Staff', 'Standard staff role')
        "#,
    )
    .bind(owner_role_id)
    .bind(store_id)
    .bind(staff_role_id)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    sqlx::query(
        r#"
        INSERT INTO store_sync_settings (store_id, tenant_id, customer_sync_enabled)
        VALUES ($1, $2, true)
        ON CONFLICT (store_id) DO NOTHING
        "#,
    )
    .bind(store_id)
    .bind(tenant_id)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    let vendor_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO vendors (id, tenant_id, name, commission_rate, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(vendor_id)
    .bind(tenant_id)
    .bind(&req.store_name)
    .bind(0.0)
    .bind("active")
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    let password_hash = hash_password(&req.owner_password)?;
    let owner_staff_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO store_staff (id, store_id, email, login_id, phone, password_hash, role_id, status)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        "#,
    )
    .bind(owner_staff_id)
    .bind(store_id)
    .bind(&req.owner_email)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(password_hash)
    .bind(owner_role_id)
    .bind("active")
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    let actor_id = req.actor.as_ref().and_then(|actor| {
        if actor.actor_id.is_empty() {
            None
        } else {
            Some(actor.actor_id.clone())
        }
    });
    let actor_type = req
        .actor
        .as_ref()
        .and_then(|actor| {
            if actor.actor_type.is_empty() {
                None
            } else {
                Some(actor.actor_type.clone())
            }
        })
        .unwrap_or_else(|| "system".to_string());

    audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(store_id.to_string()),
            actor_id: actor_id.clone(),
            actor_type: actor_type.clone(),
            action: StoreSettingsAuditAction::Initialize.into(),
            target_type: Some("store_settings".to_string()),
            target_id: Some(store_id.to_string()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: Some(serde_json::json!({
                "store_id": store_id.to_string(),
                "store_name": req.store_name,
                "store_code": store_code.as_str(),
            })),
            metadata_json: None,
        },
    )
    .await?;

    audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(store_id.to_string()),
            actor_id: actor_id.clone(),
            actor_type: actor_type.clone(),
            action: MallSettingsAuditAction::Initialize.into(),
            target_type: Some("mall_settings".to_string()),
            target_id: Some(store_id.to_string()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: Some(serde_json::json!({
                "store_id": store_id.to_string(),
            })),
            metadata_json: None,
        },
    )
    .await?;

    audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(store_id.to_string()),
            actor_id,
            actor_type,
            action: IdentityAuditAction::StaffCreate.into(),
            target_type: Some("store_staff".to_string()),
            target_id: Some(owner_staff_id.to_string()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: Some(serde_json::json!({
                "staff_id": owner_staff_id.to_string(),
                "store_id": store_id.to_string(),
                "role_id": owner_role_id.to_string(),
                "role_key": "owner",
                "email": req.owner_email,
            })),
            metadata_json: None,
        },
    )
    .await?;

    tx.commit().await.map_err(db::error)?;

    // Ensure search settings exist (safe to call repeatedly).
    let _ = state.search.ensure_settings().await;

    Ok(pb::InitializeStoreResponse {
        tenant_id: tenant_id.to_string(),
        store_id: store_id.to_string(),
        owner_staff_id: owner_staff_id.to_string(),
        vendor_id: vendor_id.to_string(),
        store_code: store_code.as_str().to_string(),
    })
}

pub async fn validate_store_code(
    state: &AppState,
    req: pb::ValidateStoreCodeRequest,
) -> Result<pb::ValidateStoreCodeResponse, (StatusCode, Json<ConnectError>)> {
    let store_code = match StoreCode::parse(&req.store_code) {
        Ok(value) => value,
        Err((_, Json(err))) => {
            return Ok(pb::ValidateStoreCodeResponse {
                available: false,
                message: err.message,
            });
        }
    };

    let existing = sqlx::query("SELECT 1 FROM stores WHERE code = $1 LIMIT 1")
        .bind(store_code.as_str())
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?;
    if existing.is_some() {
        return Ok(pb::ValidateStoreCodeResponse {
            available: false,
            message: "store_code already exists".to_string(),
        });
    }

    Ok(pb::ValidateStoreCodeResponse {
        available: true,
        message: "store_code is available".to_string(),
    })
}

fn hash_password(password: &str) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::Internal,
                    message: "failed to hash password".to_string(),
                }),
            )
        })
}
