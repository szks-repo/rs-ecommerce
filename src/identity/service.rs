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
use chrono::{Duration, Utc};

use crate::{
    AppState,
    pb::pb,
    infrastructure::{db, audit, email},
    rpc::json::ConnectError,
    shared::validation::{Email, Phone},
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
    let email = Email::parse_optional(&req.email)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    let login_id = req.login_id.clone();
    let phone = Phone::parse_optional(&req.phone)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
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

pub async fn list_staff(
    state: &AppState,
    req: pb::IdentityListStaffRequest,
) -> Result<pb::IdentityListStaffResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;

    let rows = sqlx::query(
        r#"
        SELECT ss.id::text as staff_id, ss.email, ss.login_id, ss.phone, ss.status,
               ss.display_name, sr.id::text as role_id, sr.key as role_key
        FROM store_staff ss
        JOIN store_roles sr ON sr.id = ss.role_id
        WHERE ss.store_id = $1
        ORDER BY ss.created_at ASC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    let staff = rows
        .into_iter()
        .map(|row| pb::IdentityStaffSummary {
            staff_id: row.get("staff_id"),
            email: row.get::<Option<String>, _>("email").unwrap_or_default(),
            login_id: row.get::<Option<String>, _>("login_id").unwrap_or_default(),
            phone: row.get::<Option<String>, _>("phone").unwrap_or_default(),
            role_id: row.get("role_id"),
            role_key: row.get("role_key"),
            status: row.get("status"),
            display_name: row.get::<Option<String>, _>("display_name").unwrap_or_default(),
        })
        .collect();

    Ok(pb::IdentityListStaffResponse { staff })
}

pub async fn update_staff(
    state: &AppState,
    req: pb::IdentityUpdateStaffRequest,
) -> Result<pb::IdentityUpdateStaffResponse, (StatusCode, Json<ConnectError>)> {
    if req.staff_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "staff_id is required".to_string(),
            }),
        ));
    }
    if req.role_id.is_empty() && req.status.is_empty() && req.display_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "role_id, status, or display_name is required".to_string(),
            }),
        ));
    }

    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let staff_uuid = parse_uuid(&req.staff_id, "staff_id")?;

    let current = sqlx::query(
        r#"
        SELECT sr.key as role_key
        FROM store_staff ss
        JOIN store_roles sr ON sr.id = ss.role_id
        WHERE ss.id = $1 AND ss.store_id = $2
        "#,
    )
    .bind(staff_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    if let Some(row) = current {
        let current_role: String = row.get("role_key");
        if current_role == "owner" && !req.role_id.is_empty() {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "owner role cannot be changed".to_string(),
                }),
            ));
        }
    }

    if !req.role_id.is_empty() {
        let role_uuid = parse_uuid(&req.role_id, "role_id")?;
        let role_row = sqlx::query(
            r#"
            SELECT key
            FROM store_roles
            WHERE id = $1 AND store_id = $2
            "#,
        )
        .bind(role_uuid)
        .bind(store_uuid.as_uuid())
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?;
        let Some(role_row) = role_row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "role_id is invalid".to_string(),
                }),
            ));
        };
        let role_key: String = role_row.get("key");
        if role_key == "owner" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "owner role cannot be assigned".to_string(),
                }),
            ));
        }
    }

    let updated = sqlx::query(
        r#"
        UPDATE store_staff
        SET role_id = COALESCE(NULLIF($1, '')::uuid, role_id),
            status = COALESCE(NULLIF($2, ''), status),
            display_name = COALESCE(NULLIF($3, ''), display_name),
            updated_at = now()
        WHERE id = $4 AND store_id = $5
        RETURNING id::text as staff_id, email, login_id, phone, status, role_id::text as role_id, display_name
        "#,
    )
    .bind(&req.role_id)
    .bind(&req.status)
    .bind(&req.display_name)
    .bind(staff_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    let Some(row) = updated else {
        return Ok(pb::IdentityUpdateStaffResponse {
            updated: false,
            staff: None,
        });
    };

    let role_key_row = sqlx::query(
        r#"
        SELECT key
        FROM store_roles
        WHERE id = $1 AND store_id = $2
        "#,
    )
    .bind(parse_uuid(&row.get::<String, _>("role_id"), "role_id")?)
    .bind(store_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    let staff = pb::IdentityStaffSummary {
        staff_id: row.get("staff_id"),
        email: row.get::<Option<String>, _>("email").unwrap_or_default(),
        login_id: row.get::<Option<String>, _>("login_id").unwrap_or_default(),
        phone: row.get::<Option<String>, _>("phone").unwrap_or_default(),
        role_id: row.get("role_id"),
        role_key: role_key_row.get("key"),
        status: row.get("status"),
        display_name: row.get::<Option<String>, _>("display_name").unwrap_or_default(),
    };

    let actor_id = req
        .actor
        .as_ref()
        .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
    if actor_id.is_none() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::PermissionDenied,
                message: "actor_id is required".to_string(),
            }),
        ));
    }
    let actor_type = req
        .actor
        .as_ref()
        .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record(
        state,
        audit::AuditInput {
            tenant_id,
            actor_id,
            actor_type,
            action: IdentityAuditAction::StaffUpdate.into(),
            target_type: Some("store_staff".to_string()),
            target_id: Some(staff.staff_id.clone()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: Some(serde_json::json!({
                "staff_id": staff.staff_id,
                "role_id": staff.role_id,
                "role_key": staff.role_key,
                "status": staff.status,
                "display_name": staff.display_name,
            })),
            metadata_json: None,
        },
    )
    .await?;

    Ok(pb::IdentityUpdateStaffResponse {
        updated: true,
        staff: Some(staff),
    })
}

pub async fn invite_staff(
    state: &AppState,
    req: pb::IdentityInviteStaffRequest,
) -> Result<pb::IdentityInviteStaffResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    require_owner(req.actor.as_ref())?;

    let invite_email = Email::parse(&req.email)?;
    if req.role_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "role_id is required".to_string(),
            }),
        ));
    }

    let store_uuid = StoreId::parse(&store_id)?;
    let role_uuid = parse_uuid(&req.role_id, "role_id")?;
    let role_row = sqlx::query(
        r#"
        SELECT key, name
        FROM store_roles
        WHERE id = $1 AND store_id = $2
        "#,
    )
    .bind(role_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    let Some(role_row) = role_row else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "role_id is invalid".to_string(),
            }),
        ));
    };
    let role_key: String = role_row.get("key");
    let role_name: Option<String> = role_row.get("name");
    if role_key == "owner" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "owner role cannot be invited".to_string(),
            }),
        ));
    }

    let existing = sqlx::query(
        r#"
        SELECT 1
        FROM store_staff
        WHERE store_id = $1 AND email = $2
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(invite_email.as_str())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::AlreadyExists,
                message: "email already exists".to_string(),
            }),
        ));
    }

    let existing_invite = sqlx::query(
        r#"
        SELECT 1
        FROM store_staff_invites
        WHERE store_id = $1 AND email = $2 AND accepted_at IS NULL
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(invite_email.as_str())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    if existing_invite.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::AlreadyExists,
                message: "invite already exists".to_string(),
            }),
        ));
    }

    let actor_id = req
        .actor
        .as_ref()
        .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
    let created_by = match actor_id.as_ref() {
        Some(id) => Some(parse_uuid(id, "actor_id")?),
        None => None,
    };

    let mut tx = state.db.begin().await.map_err(db::error)?;
    let staff_id = uuid::Uuid::new_v4();
    let invite_id = uuid::Uuid::new_v4();
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query(
        r#"
        INSERT INTO store_staff (id, store_id, email, role_id, status, display_name)
        VALUES ($1,$2,$3,$4,$5,$6)
        "#,
    )
    .bind(staff_id)
    .bind(store_uuid.as_uuid())
    .bind(invite_email.as_str())
    .bind(role_uuid)
    .bind("invited")
    .bind(if req.display_name.is_empty() { None } else { Some(req.display_name.clone()) })
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    sqlx::query(
        r#"
        INSERT INTO store_staff_invites (id, store_id, email, role_id, token, created_by, expires_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        "#,
    )
    .bind(invite_id)
    .bind(store_uuid.as_uuid())
    .bind(invite_email.as_str())
    .bind(role_uuid)
    .bind(&token)
    .bind(created_by)
    .bind(expires_at)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    tx.commit().await.map_err(db::error)?;

    let store_row = sqlx::query("SELECT name FROM stores WHERE id = $1")
        .bind(store_uuid.as_uuid())
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?;
    let store_name: String = store_row
        .and_then(|row| row.try_get::<String, _>("name").ok())
        .unwrap_or_else(|| "Store".to_string());

    let email_config = email::EmailConfig::from_env();
    email::send_invite_email(
        &email_config,
        invite_email.as_str(),
        &store_name,
        Some(req.display_name.as_str()),
        role_name.as_deref(),
        &token,
    )
    .await?;

    let _ = audit::record(
        state,
        audit::AuditInput {
            tenant_id,
            actor_id,
            actor_type: req
                .actor
                .as_ref()
                .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
                .unwrap_or_else(|| "staff".to_string()),
            action: IdentityAuditAction::StaffInvite.into(),
            target_type: Some("store_staff_invite".to_string()),
            target_id: Some(invite_id.to_string()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: Some(serde_json::json!({
                "invite_id": invite_id.to_string(),
                "staff_id": staff_id.to_string(),
                "email": invite_email.as_str(),
                "role_id": req.role_id,
            })),
            metadata_json: None,
        },
    )
    .await?;

    Ok(pb::IdentityInviteStaffResponse {
        invite_id: invite_id.to_string(),
        invite_token: token,
        email: invite_email.as_str().to_string(),
        role_id: req.role_id,
        expires_at: chrono_to_timestamp(Some(expires_at)),
    })
}

pub async fn transfer_owner(
    state: &AppState,
    req: pb::IdentityTransferOwnerRequest,
) -> Result<pb::IdentityTransferOwnerResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    require_owner(req.actor.as_ref())?;

    if req.new_owner_staff_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "new_owner_staff_id is required".to_string(),
            }),
        ));
    }

    let store_uuid = StoreId::parse(&store_id)?;
    let new_owner_uuid = parse_uuid(&req.new_owner_staff_id, "new_owner_staff_id")?;
    let actor_id = req
        .actor
        .as_ref()
        .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });

    let current_owner_row = sqlx::query(
        r#"
        SELECT ss.id::text as staff_id
        FROM store_staff ss
        JOIN store_roles sr ON sr.id = ss.role_id
        WHERE ss.store_id = $1 AND sr.key = 'owner'
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    let Some(current_owner_row) = current_owner_row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::NotFound,
                message: "owner not found".to_string(),
            }),
        ));
    };
    let current_owner_id: String = current_owner_row.get("staff_id");
    if current_owner_id == req.new_owner_staff_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "new owner is already the owner".to_string(),
            }),
        ));
    }

    if let Some(actor_id) = actor_id.as_ref() {
        if actor_id != &current_owner_id {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "only owner can transfer ownership".to_string(),
                }),
            ));
        }
    }

    let target_row = sqlx::query(
        r#"
        SELECT ss.id::text as staff_id, ss.status
        FROM store_staff ss
        WHERE ss.id = $1 AND ss.store_id = $2
        "#,
    )
    .bind(new_owner_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    let Some(target_row) = target_row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::NotFound,
                message: "new owner staff not found".to_string(),
            }),
        ));
    };
    let target_status: String = target_row.get("status");
    if target_status != "active" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "new owner must be active".to_string(),
            }),
        ));
    }

    let owner_role_row = sqlx::query(
        "SELECT id FROM store_roles WHERE store_id = $1 AND key = 'owner'",
    )
    .bind(store_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    let owner_role_id: uuid::Uuid = owner_role_row.get("id");

    let staff_role_row = sqlx::query(
        "SELECT id FROM store_roles WHERE store_id = $1 AND key = 'staff'",
    )
    .bind(store_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    let staff_role_id: uuid::Uuid = staff_role_row.get("id");

    let mut tx = state.db.begin().await.map_err(db::error)?;
    sqlx::query(
        r#"
        UPDATE store_staff
        SET role_id = $1, updated_at = now()
        WHERE id = $2 AND store_id = $3
        "#,
    )
    .bind(owner_role_id)
    .bind(new_owner_uuid)
    .bind(store_uuid.as_uuid())
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    sqlx::query(
        r#"
        UPDATE store_staff
        SET role_id = $1, updated_at = now()
        WHERE id = $2 AND store_id = $3
        "#,
    )
    .bind(staff_role_id)
    .bind(parse_uuid(&current_owner_id, "current_owner_id")?)
    .bind(store_uuid.as_uuid())
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    tx.commit().await.map_err(db::error)?;

    let new_owner = fetch_staff_summary(state, &store_uuid.as_uuid(), &req.new_owner_staff_id).await?;
    let previous_owner = fetch_staff_summary(state, &store_uuid.as_uuid(), &current_owner_id).await?;

    let _ = audit::record(
        state,
        audit::AuditInput {
            tenant_id,
            actor_id,
            actor_type: req
                .actor
                .as_ref()
                .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
                .unwrap_or_else(|| "staff".to_string()),
            action: IdentityAuditAction::OwnerTransfer.into(),
            target_type: Some("store".to_string()),
            target_id: Some(store_id),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: Some(serde_json::json!({ "owner_staff_id": current_owner_id })),
            after_json: Some(serde_json::json!({ "owner_staff_id": req.new_owner_staff_id })),
            metadata_json: None,
        },
    )
    .await?;

    Ok(pb::IdentityTransferOwnerResponse {
        transferred: true,
        new_owner: Some(new_owner),
        previous_owner: Some(previous_owner),
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
    let store = req.store.clone();
    let tenant = req.tenant.clone();
    let email = req.email.clone();
    let login_id = req.login_id.clone();
    let phone = req.phone.clone();
    let role_id = req.role_id.clone();
    let display_name = req.display_name.clone();
    let actor = req.actor.clone();

    let resp = create_staff_core(
        state,
        store,
        tenant,
        email.clone(),
        login_id.clone(),
        phone.clone(),
        req.password,
        role_id.clone(),
        display_name.clone(),
    )
    .await?;

    let actor_id = actor
        .as_ref()
        .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
    let actor_type = actor
        .as_ref()
        .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record(
        state,
        audit::AuditInput {
            tenant_id: resp.tenant_id.clone(),
            actor_id,
            actor_type,
            action: IdentityAuditAction::StaffCreate.into(),
            target_type: Some("store_staff".to_string()),
            target_id: Some(resp.staff_id.clone()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: Some(serde_json::json!({
                "staff_id": resp.staff_id,
                "store_id": resp.store_id,
                "role_id": resp.role_id,
                "role_key": resp.role_key,
                "email": email,
                "login_id": login_id,
                "phone": phone,
                "display_name": display_name,
            })),
            metadata_json: None,
        },
    )
    .await?;

    Ok(pb::IdentityCreateStaffResponse {
        staff_id: resp.staff_id,
        store_id: resp.store_id,
        tenant_id: resp.tenant_id,
        role_id: resp.role_id,
        role_key: resp.role_key,
        display_name: resp.display_name,
    })
}


pub async fn create_role(
    state: &AppState,
    req: pb::IdentityCreateRoleRequest,
) -> Result<pb::IdentityCreateRoleResponse, (StatusCode, Json<ConnectError>)> {
    let store = req.store.clone();
    let tenant = req.tenant.clone();
    let key = req.key.clone();
    let name = req.name.clone();
    let description = req.description.clone();
    let permissions = req.permission_keys.clone();
    let actor = req.actor.clone();

    let resp = create_role_core(
        state,
        store,
        tenant,
        key.clone(),
        name.clone(),
        description.clone(),
        permissions,
    )
    .await?;

    if let Some(role) = resp.role.as_ref() {
        let actor_id = actor
            .as_ref()
            .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
        let actor_type = actor
            .as_ref()
            .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
            .unwrap_or_else(|| "staff".to_string());
        let tenant_id = resolve_store_context(state, req.store, req.tenant)
            .await
            .map(|(_, tenant_id)| tenant_id)?;

        let _ = audit::record(
            state,
            audit::AuditInput {
                tenant_id,
                actor_id,
                actor_type,
                action: IdentityAuditAction::RoleCreate.into(),
                target_type: Some("store_role".to_string()),
                target_id: Some(role.id.clone()),
                request_id: None,
                ip_address: None,
                user_agent: None,
                before_json: None,
                after_json: Some(serde_json::json!({
                    "role_id": role.id,
                    "key": role.key,
                    "name": role.name,
                })),
                metadata_json: None,
            },
        )
        .await?;
    }

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
    let store = req.store.clone();
    let tenant = req.tenant.clone();
    let staff_id = req.staff_id.clone();
    let role_id = req.role_id.clone();
    let actor = req.actor.clone();

    let resp = assign_role_to_staff_core(
        state,
        store,
        tenant,
        staff_id.clone(),
        role_id.clone(),
    )
    .await?;

    if resp.assigned {
        let actor_id = actor
            .as_ref()
            .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
        let actor_type = actor
            .as_ref()
            .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
            .unwrap_or_else(|| "staff".to_string());
        let tenant_id = resolve_store_context(state, req.store, req.tenant)
            .await
            .map(|(_, tenant_id)| tenant_id)?;

        let _ = audit::record(
            state,
            audit::AuditInput {
                tenant_id,
                actor_id,
                actor_type,
                action: IdentityAuditAction::RoleAssign.into(),
                target_type: Some("store_staff".to_string()),
                target_id: Some(staff_id),
                request_id: None,
                ip_address: None,
                user_agent: None,
                before_json: None,
                after_json: None,
                metadata_json: Some(serde_json::json!({
                    "role_id": role_id,
                })),
            },
        )
        .await?;
    }

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

pub async fn list_roles_with_permissions(
    state: &AppState,
    req: pb::IdentityListRolesWithPermissionsRequest,
) -> Result<pb::IdentityListRolesWithPermissionsResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;

    let role_rows = sqlx::query(
        r#"
        SELECT id, id::text as id_text, key, name, description
        FROM store_roles
        WHERE store_id = $1 AND key <> 'owner'
        ORDER BY created_at ASC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    let mut roles: Vec<pb::IdentityRoleDetail> = role_rows
        .iter()
        .map(|row| pb::IdentityRoleDetail {
            id: row.get("id_text"),
            key: row.get("key"),
            name: row.get("name"),
            description: row.get::<Option<String>, _>("description").unwrap_or_default(),
            permission_keys: Vec::new(),
        })
        .collect();

    if !roles.is_empty() {
        let role_ids: Vec<uuid::Uuid> = role_rows
            .iter()
            .map(|row| row.get::<uuid::Uuid, _>("id"))
            .collect();
        let permission_rows = sqlx::query(
            r#"
            SELECT srp.role_id::text as role_id, p.key as key
            FROM store_role_permissions srp
            JOIN permissions p ON p.id = srp.permission_id
            WHERE srp.role_id = ANY($1)
            "#,
        )
        .bind(&role_ids)
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?;

        for row in permission_rows {
            let role_id: String = row.get("role_id");
            let key: String = row.get("key");
            if let Some(role) = roles.iter_mut().find(|r| r.id == role_id) {
                role.permission_keys.push(key);
            }
        }
    }

    Ok(pb::IdentityListRolesWithPermissionsResponse { roles })
}

pub async fn update_role(
    state: &AppState,
    req: pb::IdentityUpdateRoleRequest,
) -> Result<pb::IdentityUpdateRoleResponse, (StatusCode, Json<ConnectError>)> {
    if req.role_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "role_id is required".to_string(),
            }),
        ));
    }

    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let role_uuid = parse_uuid(&req.role_id, "role_id")?;
    let permission_keys = req.permission_keys.clone();

    let rows = sqlx::query(
        r#"
        SELECT id::text as id, key
        FROM permissions
        WHERE key = ANY($1)
        "#,
    )
    .bind(&permission_keys)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    if rows.len() != permission_keys.len() && !permission_keys.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "unknown permission key included".to_string(),
            }),
        ));
    }

    let mut tx = state.db.begin().await.map_err(db::error)?;
    let updated = sqlx::query(
        r#"
        UPDATE store_roles
        SET name = COALESCE(NULLIF($1, ''), name),
            description = COALESCE(NULLIF($2, ''), description),
            updated_at = now()
        WHERE id = $3 AND store_id = $4
        RETURNING id::text as id, key, name, description
        "#,
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(role_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::error)?;

    let Some(row) = updated else {
        return Ok(pb::IdentityUpdateRoleResponse {
            updated: false,
            role: None,
        });
    };

    sqlx::query("DELETE FROM store_role_permissions WHERE role_id = $1")
        .bind(role_uuid)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;

    for permission in rows {
        let permission_id: String = permission.get("id");
        sqlx::query(
            r#"
            INSERT INTO store_role_permissions (role_id, permission_id)
            VALUES ($1,$2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(role_uuid)
        .bind(parse_uuid(&permission_id, "permission_id")?)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    }

    tx.commit().await.map_err(db::error)?;

    let role = pb::IdentityRoleDetail {
        id: row.get("id"),
        key: row.get("key"),
        name: row.get("name"),
        description: row.get::<Option<String>, _>("description").unwrap_or_default(),
        permission_keys,
    };

    let actor_id = req
        .actor
        .as_ref()
        .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
    let actor_type = req
        .actor
        .as_ref()
        .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record(
        state,
        audit::AuditInput {
            tenant_id,
            actor_id,
            actor_type,
            action: IdentityAuditAction::RoleUpdate.into(),
            target_type: Some("store_role".to_string()),
            target_id: Some(role.id.clone()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: Some(serde_json::json!({
                "role_id": role.id,
                "key": role.key,
                "name": role.name,
                "description": role.description,
                "permission_keys": role.permission_keys,
            })),
            metadata_json: None,
        },
    )
    .await?;

    Ok(pb::IdentityUpdateRoleResponse {
        updated: true,
        role: Some(role),
    })
}

pub async fn delete_role(
    state: &AppState,
    req: pb::IdentityDeleteRoleRequest,
) -> Result<pb::IdentityDeleteRoleResponse, (StatusCode, Json<ConnectError>)> {
    if req.role_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "role_id is required".to_string(),
            }),
        ));
    }

    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let role_uuid = parse_uuid(&req.role_id, "role_id")?;

    let attached = sqlx::query(
        r#"
        SELECT 1
        FROM store_staff
        WHERE role_id = $1
        LIMIT 1
        "#,
    )
    .bind(role_uuid)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    if attached.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::FailedPrecondition,
                message: "role is attached to staff".to_string(),
            }),
        ));
    }

    let deleted = sqlx::query(
        r#"
        DELETE FROM store_roles
        WHERE id = $1 AND store_id = $2
        RETURNING id::text as id, key, name
        "#,
    )
    .bind(role_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    if let Some(row) = deleted {
        let actor_id = req
            .actor
            .as_ref()
            .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
        let actor_type = req
            .actor
            .as_ref()
            .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.clone()) })
            .unwrap_or_else(|| "staff".to_string());

        let _ = audit::record(
            state,
            audit::AuditInput {
                tenant_id,
                actor_id,
                actor_type,
                action: IdentityAuditAction::RoleDelete.into(),
                target_type: Some("store_role".to_string()),
                target_id: Some(row.get::<String, _>("id")),
                request_id: None,
                ip_address: None,
                user_agent: None,
                before_json: Some(serde_json::json!({
                    "role_id": row.get::<String, _>("id"),
                    "key": row.get::<String, _>("key"),
                    "name": row.get::<String, _>("name"),
                })),
                after_json: None,
                metadata_json: None,
            },
        )
        .await?;

        return Ok(pb::IdentityDeleteRoleResponse { deleted: true });
    }

    Ok(pb::IdentityDeleteRoleResponse { deleted: false })
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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "password is required".to_string(),
            }),
        ));
    }

    let email = Email::parse_optional(&email)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    let phone = Phone::parse_optional(&phone)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();

    let (store_id, tenant_id) = resolve_store_context_without_token_guard(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;

    let row = if !email.is_empty() {
        sqlx::query(
            r#"
            SELECT ss.id::text as id, ss.email, ss.login_id, ss.phone, ss.password_hash,
                   sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.store_id = $1 AND ss.email = $2 AND ss.status = 'active'
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
            SELECT ss.id::text as id, ss.email, ss.login_id, ss.phone, ss.password_hash,
                   sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.store_id = $1 AND ss.login_id = $2 AND ss.status = 'active'
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
            SELECT ss.id::text as id, ss.email, ss.login_id, ss.phone, ss.password_hash,
                   sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.store_id = $1 AND ss.phone = $2 AND ss.status = 'active'
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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "email or login_id or phone is required".to_string(),
            }),
        ));
    };

    let Some(row) = row else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::Unauthenticated,
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
                    code: crate::rpc::json::ErrorCode::Unauthenticated,
                    message: "invalid credentials".to_string(),
                }),
            )
        })?;

    let parsed_hash = PasswordHash::new(&hash).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::Unauthenticated,
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
                    code: crate::rpc::json::ErrorCode::Unauthenticated,
                    message: "invalid credentials".to_string(),
                }),
            )
        })?;

    let staff_id: String = row.get("id");
    let role: String = row.get("role_key");

    let jwt_secret = std::env::var("AUTH_JWT_SECRET").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::Internal,
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
                code: crate::rpc::json::ErrorCode::Internal,
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
    role_id: String,
    role_key: String,
    display_name: String,
}

async fn create_staff_core(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    email: String,
    login_id: String,
    phone: String,
    password: String,
    role_id: String,
    display_name: String,
) -> Result<CreateStaffCoreResult, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;
    let email = Email::parse_optional(&email)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    let phone = Phone::parse_optional(&phone)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();

    if role_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "role_id is required".to_string(),
            }),
        ));
    }
    if email.is_empty() && login_id.is_empty() && phone.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "email or login_id or phone is required".to_string(),
            }),
        ));
    }
    if password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "password is required".to_string(),
            }),
        ));
    }

    let password_hash = hash_password(&password)?;
    let staff_id = uuid::Uuid::new_v4();

    let role_uuid = parse_uuid(&role_id, "role_id")?;
    let role_row = sqlx::query(
        r#"
        SELECT key
        FROM store_roles
        WHERE id = $1 AND store_id = $2
        "#,
    )
    .bind(role_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    let role_key: String = role_row.get("key");
    if role_key == "owner" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "owner role cannot be assigned".to_string(),
            }),
        ));
    }

    sqlx::query(
        r#"
        INSERT INTO store_staff (id, store_id, email, login_id, phone, password_hash, role_id, status, display_name)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        "#,
    )
    .bind(staff_id)
    .bind(store_uuid.as_uuid())
    .bind(if email.is_empty() { None } else { Some(email.clone()) })
    .bind(if login_id.is_empty() { None } else { Some(login_id) })
    .bind(if phone.is_empty() { None } else { Some(phone.clone()) })
    .bind(password_hash)
    .bind(role_uuid)
    .bind("active")
    .bind(if display_name.is_empty() { None } else { Some(display_name.clone()) })
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    Ok(CreateStaffCoreResult {
        staff_id: staff_id.to_string(),
        store_id,
        tenant_id,
        role_id,
        role_key,
        display_name,
    })
}

fn require_owner(actor: Option<&pb::ActorContext>) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let actor_type = actor
        .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type.as_str()) })
        .unwrap_or("");
    if actor_type != "owner" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::PermissionDenied,
                message: "owner role is required".to_string(),
            }),
        ));
    }
    Ok(())
}

async fn fetch_staff_summary(
    state: &AppState,
    store_uuid: &uuid::Uuid,
    staff_id: &str,
) -> Result<pb::IdentityStaffSummary, (StatusCode, Json<ConnectError>)> {
    let staff_uuid = parse_uuid(staff_id, "staff_id")?;
    let row = sqlx::query(
        r#"
        SELECT ss.id::text as staff_id, ss.email, ss.login_id, ss.phone, ss.status,
               ss.display_name, sr.id::text as role_id, sr.key as role_key
        FROM store_staff ss
        JOIN store_roles sr ON sr.id = ss.role_id
        WHERE ss.id = $1 AND ss.store_id = $2
        "#,
    )
    .bind(staff_uuid)
    .bind(store_uuid)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::IdentityStaffSummary {
        staff_id: row.get("staff_id"),
        email: row.get::<Option<String>, _>("email").unwrap_or_default(),
        login_id: row.get::<Option<String>, _>("login_id").unwrap_or_default(),
        phone: row.get::<Option<String>, _>("phone").unwrap_or_default(),
        role_id: row.get("role_id"),
        role_key: row.get("role_key"),
        status: row.get("status"),
        display_name: row.get::<Option<String>, _>("display_name").unwrap_or_default(),
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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "staff_id and role_id are required".to_string(),
            }),
        ));
    }

    let store_uuid = StoreId::parse(&store_id)?;
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
                code: crate::rpc::json::ErrorCode::PermissionDenied,
                message: "role does not belong to store".to_string(),
            }),
        ));
    }

    let staff_row = sqlx::query(
        r#"
        SELECT sr.key as role_key
        FROM store_staff ss
        JOIN store_roles sr ON sr.id = ss.role_id
        WHERE ss.id = $1 AND ss.store_id = $2
        "#,
    )
    .bind(parse_uuid(&staff_id, "staff_id")?)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    if let Some(row) = staff_row {
        let current_role: String = row.get("role_key");
        if current_role == "owner" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "owner role cannot be assigned".to_string(),
                }),
            ));
        }
    }

    sqlx::query(
        r#"
        UPDATE store_staff
        SET role_id = $1, updated_at = now()
        WHERE id = $2 AND store_id = $3
        "#,
    )
    .bind(parse_uuid(&role_id, "role_id")?)
    .bind(parse_uuid(&staff_id, "staff_id")?)
    .bind(store_uuid.as_uuid())
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
        WHERE store_id = $1 AND key <> 'owner'
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
