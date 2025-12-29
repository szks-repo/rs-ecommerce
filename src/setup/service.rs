use axum::{Json, http::StatusCode};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use rand_core::OsRng;
use crate::{
    AppState,
    pb::pb,
    infrastructure::db,
    rpc::json::ConnectError,
};

pub async fn initialize_store(
    state: &AppState,
    req: pb::InitializeStoreRequest,
) -> Result<pb::InitializeStoreResponse, (StatusCode, Json<ConnectError>)> {
    let _actor = req.actor.clone();
    if req.store_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "store_name is required".to_string(),
            }),
        ));
    }
    if req.owner_email.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "owner_email is required".to_string(),
            }),
        ));
    }
    if req.owner_password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "owner_password is required".to_string(),
            }),
        ));
    }
    if req.store_code.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "store_code is required".to_string(),
            }),
        ));
    }

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
                code: "already_exists",
                message: "store already exists".to_string(),
            }),
        ));
    }

    let existing_code = sqlx::query("SELECT 1 FROM stores WHERE code = $1 LIMIT 1")
        .bind(&req.store_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::error)?;
    if existing_code.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ConnectError {
                code: "already_exists",
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
    .bind(&req.store_code)
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
        INSERT INTO store_staff (id, store_id, email, login_id, phone, password_hash, role, status)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        "#,
    )
    .bind(owner_staff_id)
    .bind(store_id)
    .bind(&req.owner_email)
    .bind(if req.owner_login_id.is_empty() { None } else { Some(req.owner_login_id.clone()) })
    .bind(Option::<String>::None)
    .bind(password_hash)
    .bind("owner")
    .bind("active")
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    tx.commit().await.map_err(db::error)?;

    // Ensure search settings exist (safe to call repeatedly).
    let _ = state.search.ensure_settings().await;

    Ok(pb::InitializeStoreResponse {
        tenant_id: tenant_id.to_string(),
        store_id: store_id.to_string(),
        owner_staff_id: owner_staff_id.to_string(),
        vendor_id: vendor_id.to_string(),
        store_code: req.store_code,
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
                    code: "internal",
                    message: "failed to hash password".to_string(),
                }),
            )
        })
}
