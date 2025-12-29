use axum::{Json, http::StatusCode};
use argon2::{
    Argon2,
    PasswordHash,
    PasswordVerifier,
    password_hash::{PasswordHasher, SaltString},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand_core::OsRng;
use serde::Serialize;
use sqlx::Row;

use crate::{
    AppState,
    pb::pb,
    infrastructure::{db, audit},
    rpc::json::ConnectError,
    shared::{
        time::chrono_to_timestamp,
        audit_action::IdentityAuditAction,
        ids::{StoreId, TenantId},
    },
    identity::context::{parse_uuid, resolve_store_context, resolve_store_context_without_token_guard},
};

pub async fn sign_in(
    state: &AppState,
    req: pb::IdentitySignInRequest,
) -> Result<pb::IdentitySignInResponse, (StatusCode, Json<ConnectError>)> {
    let email = req.email.clone();
    let login_id = req.login_id.clone();
    let phone = req.phone.clone();
    let resp = sign_in_core(
        state,
        req.store,
        req.tenant,
        email.clone(),
        login_id.clone(),
        phone.clone(),
        req.password,
    )
    .await?;

    let identifier = if !email.is_empty() {
        Some(("email", email))
    } else if !login_id.is_empty() {
        Some(("login_id", login_id))
    } else if !phone.is_empty() {
        Some(("phone", phone))
    } else {
        None
    };

    let metadata_json = identifier.map(|(key, value)| {
        serde_json::json!({
            "identifier_type": key,
            "identifier": value,
        })
    });

    let _ = audit::record(
        state,
        audit::AuditInput {
            tenant_id: resp.tenant_id.clone(),
            actor_id: Some(resp.staff_id.clone()),
            actor_type: resp.role.clone(),
            action: IdentityAuditAction::SignIn.into(),
            target_type: Some("store_staff".to_string()),
            target_id: Some(resp.staff_id.clone()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: None,
            metadata_json,
        },
    )
    .await?;

    Ok(pb::IdentitySignInResponse {
        access_token: resp.access_token,
        store_id: resp.store_id,
        tenant_id: resp.tenant_id,
        staff_id: resp.staff_id,
        role: resp.role,
        expires_at: resp.expires_at,
    })
}

pub async fn sign_out(
    state: &AppState,
    req: pb::IdentitySignOutRequest,
    actor: Option<pb::ActorContext>,
) -> Result<pb::IdentitySignOutResponse, (StatusCode, Json<ConnectError>)> {
    let (_store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let actor_id = actor.as_ref().and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
    let actor_type = actor
        .as_ref()
        .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record(
        state,
        audit::AuditInput {
            tenant_id,
            actor_id,
            actor_type,
            action: IdentityAuditAction::SignOut.into(),
            target_type: None,
            target_id: None,
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: None,
            metadata_json: None,
        },
    )
    .await?;

    Ok(pb::IdentitySignOutResponse { signed_out: true })
}


pub async fn create_staff(
    state: &AppState,
    req: pb::IdentityCreateStaffRequest,
) -> Result<pb::IdentityCreateStaffResponse, (StatusCode, Json<ConnectError>)> {
    let resp = create_staff_core(
        state,
        req.store,
        req.tenant,
        req.email,
        req.login_id,
        req.phone,
        req.password,
        req.role,
    )
    .await?;

    Ok(pb::IdentityCreateStaffResponse {
        staff_id: resp.staff_id,
        store_id: resp.store_id,
        tenant_id: resp.tenant_id,
        role: resp.role,
    })
}


pub async fn create_role(
    state: &AppState,
    req: pb::IdentityCreateRoleRequest,
) -> Result<pb::IdentityCreateRoleResponse, (StatusCode, Json<ConnectError>)> {
    let resp = create_role_core(
        state,
        req.store,
        req.tenant,
        req.key,
        req.name,
        req.description,
        req.permission_keys,
    )
    .await?;

    Ok(pb::IdentityCreateRoleResponse {
        role: resp.role.map(|role| pb::IdentityRole {
            id: role.id,
            key: role.key,
            name: role.name,
            description: role.description,
        }),
    })
}


pub async fn assign_role_to_staff(
    state: &AppState,
    req: pb::IdentityAssignRoleRequest,
) -> Result<pb::IdentityAssignRoleResponse, (StatusCode, Json<ConnectError>)> {
    let resp = assign_role_to_staff_core(
        state,
        req.store,
        req.tenant,
        req.staff_id,
        req.role_id,
    )
    .await?;

    Ok(pb::IdentityAssignRoleResponse { assigned: resp.assigned })
}


pub async fn list_roles(
    state: &AppState,
    req: pb::IdentityListRolesRequest,
) -> Result<pb::IdentityListRolesResponse, (StatusCode, Json<ConnectError>)> {
    let resp = list_roles_core(state, req.store, req.tenant).await?;
    Ok(pb::IdentityListRolesResponse {
        roles: resp
            .roles
            .into_iter()
            .map(|role| pb::IdentityRole {
                id: role.id,
                key: role.key,
                name: role.name,
                description: role.description,
            })
            .collect(),
    })
}


struct SignInCoreResult {
    access_token: String,
    store_id: String,
    tenant_id: String,
    staff_id: String,
    role: String,
    expires_at: Option<pbjson_types::Timestamp>,
}

async fn sign_in_core(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    email: String,
    login_id: String,
    phone: String,
    password: String,
) -> Result<SignInCoreResult, (StatusCode, Json<ConnectError>)> {
    if password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "password is required".to_string(),
            }),
        ));
    }

    let (store_id, tenant_id) = resolve_store_context_without_token_guard(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;

    let row = if !email.is_empty() {
        sqlx::query(
            r#"
            SELECT id::text as id, email, login_id, phone, password_hash, role
            FROM store_staff
            WHERE store_id = $1 AND email = $2 AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(store_uuid.as_uuid())
        .bind(&email)
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?
    } else if !login_id.is_empty() {
        sqlx::query(
            r#"
            SELECT id::text as id, email, login_id, phone, password_hash, role
            FROM store_staff
            WHERE store_id = $1 AND login_id = $2 AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(store_uuid.as_uuid())
        .bind(&login_id)
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?
    } else if !phone.is_empty() {
        sqlx::query(
            r#"
            SELECT id::text as id, email, login_id, phone, password_hash, role
            FROM store_staff
            WHERE store_id = $1 AND phone = $2 AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(store_uuid.as_uuid())
        .bind(&phone)
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "email or login_id or phone is required".to_string(),
            }),
        ));
    };

    let Some(row) = row else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ConnectError {
                code: "unauthenticated",
                message: "invalid credentials".to_string(),
            }),
        ));
    };

    let hash = row
        .get::<Option<String>, _>("password_hash")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ConnectError {
                    code: "unauthenticated",
                    message: "invalid credentials".to_string(),
                }),
            )
        })?;

    let parsed_hash = PasswordHash::new(&hash).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ConnectError {
                code: "unauthenticated",
                message: "invalid credentials".to_string(),
            }),
        )
    })?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ConnectError {
                    code: "unauthenticated",
                    message: "invalid credentials".to_string(),
                }),
            )
        })?;

    let staff_id: String = row.get("id");
    let role: String = row.get("role");

    let jwt_secret = std::env::var("AUTH_JWT_SECRET").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConnectError {
                code: "internal",
                message: "AUTH_JWT_SECRET is required".to_string(),
            }),
        )
    })?;

    let now = chrono::Utc::now();
    let exp = now + chrono::Duration::hours(12);
    let claims = JwtClaims {
        sub: staff_id.clone(),
        actor_type: role.clone(),
        tenant_id: tenant_id.clone(),
        store_id: store_id.clone(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConnectError {
                code: "internal",
                message: "failed to sign token".to_string(),
            }),
        )
    })?;

    Ok(SignInCoreResult {
        access_token: token,
        store_id,
        tenant_id,
        staff_id,
        role,
        expires_at: chrono_to_timestamp(Some(exp)),
    })
}

struct CreateStaffCoreResult {
    staff_id: String,
    store_id: String,
    tenant_id: String,
    role: String,
}

async fn create_staff_core(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    email: String,
    login_id: String,
    phone: String,
    password: String,
    role: String,
) -> Result<CreateStaffCoreResult, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;

    if role.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "role is required".to_string(),
            }),
        ));
    }
    if email.is_empty() && login_id.is_empty() && phone.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "email or login_id or phone is required".to_string(),
            }),
        ));
    }
    if password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "password is required".to_string(),
            }),
        ));
    }

    let password_hash = hash_password(&password)?;
    let staff_id = uuid::Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO store_staff (id, store_id, email, login_id, phone, password_hash, role, status)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        "#,
    )
    .bind(staff_id)
    .bind(store_uuid.as_uuid())
    .bind(if email.is_empty() { None } else { Some(email) })
    .bind(if login_id.is_empty() { None } else { Some(login_id) })
    .bind(if phone.is_empty() { None } else { Some(phone) })
    .bind(password_hash)
    .bind(&role)
    .bind("active")
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    Ok(CreateStaffCoreResult {
        staff_id: staff_id.to_string(),
        store_id,
        tenant_id,
        role,
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

#[derive(Serialize)]
struct JwtClaims {
    sub: String,
    actor_type: String,
    tenant_id: String,
    store_id: String,
    exp: usize,
    iat: usize,
}

async fn create_role_core(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    key: String,
    name: String,
    description: String,
    permission_keys: Vec<String>,
) -> Result<pb::CreateRoleResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;

    if key.is_empty() || name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "key and name are required".to_string(),
            }),
        ));
    }

    let role_id = uuid::Uuid::new_v4();
    let mut tx = state.db.begin().await.map_err(db::error)?;
    sqlx::query(
        r#"
        INSERT INTO store_roles (id, store_id, key, name, description)
        VALUES ($1,$2,$3,$4,$5)
        "#,
    )
    .bind(role_id)
    .bind(store_uuid.as_uuid())
    .bind(&key)
    .bind(&name)
    .bind(&description)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    if !permission_keys.is_empty() {
        let rows = sqlx::query(
            r#"
            SELECT id::text as id, key
            FROM permissions
            WHERE key = ANY($1)
            "#,
        )
        .bind(&permission_keys)
        .fetch_all(&mut *tx)
        .await
        .map_err(db::error)?;

        for row in rows {
            let permission_id: String = row.get("id");
            sqlx::query(
                r#"
                INSERT INTO store_role_permissions (role_id, permission_id)
                VALUES ($1,$2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(role_id)
            .bind(parse_uuid(&permission_id, "permission_id")?)
            .execute(&mut *tx)
            .await
            .map_err(db::error)?;
        }
    }

    tx.commit().await.map_err(db::error)?;

    Ok(pb::CreateRoleResponse {
        role: Some(pb::Role {
            id: role_id.to_string(),
            key,
            name,
            description,
        }),
    })
}

async fn assign_role_to_staff_core(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    staff_id: String,
    role_id: String,
) -> Result<pb::AssignRoleToStaffResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    if staff_id.is_empty() || role_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "staff_id and role_id are required".to_string(),
            }),
        ));
    }

    let role_owner = sqlx::query("SELECT store_id::text as store_id FROM store_roles WHERE id = $1")
        .bind(parse_uuid(&role_id, "role_id")?)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
    let role_store_id: String = role_owner.get("store_id");
    if role_store_id != store_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ConnectError {
                code: "permission_denied",
                message: "role does not belong to store".to_string(),
            }),
        ));
    }

    sqlx::query(
        r#"
        INSERT INTO store_staff_roles (staff_id, role_id)
        VALUES ($1,$2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(parse_uuid(&staff_id, "staff_id")?)
    .bind(parse_uuid(&role_id, "role_id")?)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::AssignRoleToStaffResponse { assigned: true })
}

async fn list_roles_core(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> Result<pb::ListRolesResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id, key, name, description
        FROM store_roles
        WHERE store_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    let roles = rows
        .into_iter()
        .map(|row| pb::Role {
            id: row.get("id"),
            key: row.get("key"),
            name: row.get("name"),
            description: row.get::<Option<String>, _>("description").unwrap_or_default(),
        })
        .collect();

    Ok(pb::ListRolesResponse { roles })
}
