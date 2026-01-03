use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand_core::OsRng;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AppState,
    identity::context::{
        parse_uuid, resolve_store_context, resolve_store_context_without_token_guard,
    },
    identity::error::{IdentityError, IdentityResult},
    identity::repository::{IdentityRepository, PgIdentityRepository},
    infrastructure::{audit, email},
    pb::pb,
    shared::validation::{Email, Phone},
    shared::{
        audit_action::IdentityAuditAction,
        ids::{StoreId, TenantId},
        time::chrono_to_timestamp,
    },
};

pub struct IdentityService<'a> {
    state: &'a AppState,
}

const ACCESS_TOKEN_TTL_MINUTES: i64 = 5;
const REFRESH_TOKEN_TTL_DAYS: i64 = 30;

impl<'a> IdentityService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn sign_in(
        &self,
        req: pb::IdentitySignInRequest,
    ) -> IdentityResult<pb::IdentitySignInResponse> {
        sign_in(self.state, req).await
    }

    pub async fn sign_out(
        &self,
        req: pb::IdentitySignOutRequest,
        actor: Option<pb::ActorContext>,
    ) -> IdentityResult<pb::IdentitySignOutResponse> {
        sign_out(self.state, req, actor, None).await
    }

    pub async fn list_staff(
        &self,
        req: pb::IdentityListStaffRequest,
    ) -> IdentityResult<pb::IdentityListStaffResponse> {
        list_staff(self.state, req).await
    }

    pub async fn update_staff(
        &self,
        req: pb::IdentityUpdateStaffRequest,
    ) -> IdentityResult<pb::IdentityUpdateStaffResponse> {
        update_staff(self.state, req).await
    }

    pub async fn create_staff(
        &self,
        req: pb::IdentityCreateStaffRequest,
    ) -> IdentityResult<pb::IdentityCreateStaffResponse> {
        create_staff(self.state, req).await
    }

    pub async fn invite_staff(
        &self,
        req: pb::IdentityInviteStaffRequest,
    ) -> IdentityResult<pb::IdentityInviteStaffResponse> {
        invite_staff(self.state, req).await
    }

    pub async fn transfer_owner(
        &self,
        req: pb::IdentityTransferOwnerRequest,
    ) -> IdentityResult<pb::IdentityTransferOwnerResponse> {
        transfer_owner(self.state, req).await
    }

    pub async fn create_role(
        &self,
        req: pb::IdentityCreateRoleRequest,
    ) -> IdentityResult<pb::IdentityCreateRoleResponse> {
        create_role(self.state, req).await
    }

    pub async fn assign_role_to_staff(
        &self,
        req: pb::IdentityAssignRoleRequest,
    ) -> IdentityResult<pb::IdentityAssignRoleResponse> {
        assign_role_to_staff(self.state, req).await
    }

    pub async fn list_roles(
        &self,
        req: pb::IdentityListRolesRequest,
    ) -> IdentityResult<pb::IdentityListRolesResponse> {
        list_roles(self.state, req).await
    }

    pub async fn list_roles_with_permissions(
        &self,
        req: pb::IdentityListRolesWithPermissionsRequest,
    ) -> IdentityResult<pb::IdentityListRolesWithPermissionsResponse> {
        list_roles_with_permissions(self.state, req).await
    }

    pub async fn update_role(
        &self,
        req: pb::IdentityUpdateRoleRequest,
    ) -> IdentityResult<pb::IdentityUpdateRoleResponse> {
        update_role(self.state, req).await
    }

    pub async fn delete_role(
        &self,
        req: pb::IdentityDeleteRoleRequest,
    ) -> IdentityResult<pb::IdentityDeleteRoleResponse> {
        delete_role(self.state, req).await
    }
}

pub struct SignInWithRefresh {
    pub response: pb::IdentitySignInResponse,
    pub refresh_token: String,
}

pub async fn sign_in(
    state: &AppState,
    req: pb::IdentitySignInRequest,
) -> IdentityResult<pb::IdentitySignInResponse> {
    let result = sign_in_with_refresh(state, req).await?;
    Ok(result.response)
}

pub async fn sign_in_with_refresh(
    state: &AppState,
    req: pb::IdentitySignInRequest,
) -> IdentityResult<SignInWithRefresh> {
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
            store_id: Some(resp.store_id.clone()),
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

    Ok(SignInWithRefresh {
        response: pb::IdentitySignInResponse {
            access_token: resp.access_token,
            store_id: resp.store_id,
            tenant_id: resp.tenant_id,
            staff_id: resp.staff_id,
            role: resp.role,
            expires_at: resp.expires_at,
        },
        refresh_token: resp.refresh_token,
    })
}

pub async fn list_staff(
    state: &AppState,
    req: pb::IdentityListStaffRequest,
) -> IdentityResult<pb::IdentityListStaffResponse> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgIdentityRepository::new(&state.db);
    let staff = repo
        .list_staff(&store_uuid.as_uuid())
        .await?
        .into_iter()
        .map(|row| pb::IdentityStaffSummary {
            staff_id: row.staff_id,
            email: row.email.unwrap_or_default(),
            login_id: row.login_id.unwrap_or_default(),
            phone: row.phone.unwrap_or_default(),
            role_id: row.role_id,
            role_key: row.role_key,
            status: row.status,
            display_name: row.display_name.unwrap_or_default(),
        })
        .collect();

    Ok(pb::IdentityListStaffResponse { staff })
}

pub async fn list_staff_sessions(
    state: &AppState,
    req: pb::IdentityListStaffSessionsRequest,
) -> IdentityResult<pb::IdentityListStaffSessionsResponse> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let rows = sqlx::query(
        r#"
        SELECT s.id::text as session_id,
               ss.id::text as staff_id,
               ss.display_name,
               ss.email,
               sr.key as role_key,
               ss.status,
               s.ip_address,
               s.user_agent,
               s.last_seen_at,
               s.created_at
        FROM store_staff_sessions s
        JOIN store_staff ss ON ss.id = s.staff_id
        LEFT JOIN store_roles sr ON sr.id = ss.role_id
        WHERE s.store_id = $1
          AND s.revoked_at IS NULL
          AND (s.expires_at IS NULL OR s.expires_at > now())
        ORDER BY s.last_seen_at DESC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(IdentityError::from)?;

    let sessions = rows
        .into_iter()
        .map(|row| pb::IdentityStaffSession {
            session_id: row.get("session_id"),
            staff_id: row.get("staff_id"),
            display_name: row
                .get::<Option<String>, _>("display_name")
                .unwrap_or_default(),
            email: row.get::<Option<String>, _>("email").unwrap_or_default(),
            role_key: row.get::<Option<String>, _>("role_key").unwrap_or_default(),
            status: row.get("status"),
            ip_address: row
                .get::<Option<String>, _>("ip_address")
                .unwrap_or_default(),
            user_agent: row
                .get::<Option<String>, _>("user_agent")
                .unwrap_or_default(),
            last_seen_at: chrono_to_timestamp(Some(
                row.get::<chrono::DateTime<Utc>, _>("last_seen_at"),
            )),
            created_at: chrono_to_timestamp(Some(
                row.get::<chrono::DateTime<Utc>, _>("created_at"),
            )),
        })
        .collect();

    Ok(pb::IdentityListStaffSessionsResponse { sessions })
}

pub async fn force_sign_out_staff(
    state: &AppState,
    req: pb::IdentityForceSignOutStaffRequest,
) -> IdentityResult<pb::IdentityForceSignOutStaffResponse> {
    let (store_id, _tenant_id) =
        resolve_store_context(state, req.store.clone(), req.tenant.clone()).await?;
    if req.staff_id.is_empty() {
        return Err(IdentityError::invalid_argument("staff_id is required"));
    }
    let store_uuid = StoreId::parse(&store_id)?;
    let staff_uuid = parse_uuid(&req.staff_id, "staff_id")?;

    let result = sqlx::query(
        r#"
        UPDATE store_staff_sessions
        SET revoked_at = now()
        WHERE store_id = $1 AND staff_id = $2 AND revoked_at IS NULL
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(staff_uuid)
    .execute(&state.db)
    .await
    .map_err(IdentityError::from)?;

    let _ = sqlx::query(
        r#"
        UPDATE store_staff_refresh_tokens
        SET revoked_at = now()
        WHERE store_id = $1 AND staff_id = $2 AND revoked_at IS NULL
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(staff_uuid)
    .execute(&state.db)
    .await;

    let revoked = result.rows_affected() > 0;

    if revoked {
        let actor = req.actor.clone();
        let actor_id = actor
            .as_ref()
            .map(|a| a.actor_id.clone())
            .filter(|v| !v.is_empty());
        let actor_type = actor
            .as_ref()
            .map(|a| a.actor_type.clone())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "staff".to_string());

        let _ = audit::record(
            state,
            audit::AuditInput {
                store_id: Some(store_id),
                actor_id,
                actor_type,
                action: IdentityAuditAction::SignOut.into(),
                target_type: Some("store_staff".to_string()),
                target_id: Some(req.staff_id.clone()),
                request_id: None,
                ip_address: None,
                user_agent: None,
                before_json: None,
                after_json: None,
                metadata_json: None,
            },
        )
        .await?;
    }

    Ok(pb::IdentityForceSignOutStaffResponse { revoked })
}

pub async fn update_staff(
    state: &AppState,
    req: pb::IdentityUpdateStaffRequest,
) -> IdentityResult<pb::IdentityUpdateStaffResponse> {
    if req.staff_id.is_empty() {
        return Err(IdentityError::invalid_argument("staff_id is required"));
    }
    if req.role_id.is_empty() && req.status.is_empty() && req.display_name.is_empty() {
        return Err(IdentityError::invalid_argument(
            "role_id, status, or display_name is required",
        ));
    }

    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let staff_uuid = parse_uuid(&req.staff_id, "staff_id")?;

    let repo = PgIdentityRepository::new(&state.db);
    let current_role = repo
        .staff_role_key(&store_uuid.as_uuid(), &staff_uuid)
        .await?;
    if let Some(current_role) = current_role {
        if current_role == "owner" && !req.role_id.is_empty() {
            return Err(IdentityError::permission_denied(
                "owner role cannot be changed",
            ));
        }
    }

    if !req.role_id.is_empty() {
        let role_uuid = parse_uuid(&req.role_id, "role_id")?;
        let role_row = repo.role_by_id(&store_uuid.as_uuid(), &role_uuid).await?;
        let Some(role_row) = role_row else {
            return Err(IdentityError::invalid_argument("role_id is invalid"));
        };
        if role_row.key == "owner" {
            return Err(IdentityError::permission_denied(
                "owner role cannot be assigned",
            ));
        }
    }

    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;
    let updated = repo
        .update_staff_tx(
            tx.as_mut(),
            &staff_uuid,
            &store_uuid.as_uuid(),
            &req.role_id,
            &req.status,
            &req.display_name,
        )
        .await?;

    let Some(row) = updated else {
        tx.commit().await.map_err(IdentityError::from)?;
        return Ok(pb::IdentityUpdateStaffResponse {
            updated: false,
            staff: None,
        });
    };

    let role_key = if row.role_id.is_empty() {
        String::new()
    } else {
        sqlx::query(
            r#"
            SELECT key
            FROM store_roles
            WHERE id = $1 AND store_id = $2
            "#,
        )
        .bind(parse_uuid(&row.role_id, "role_id")?)
        .bind(store_uuid.as_uuid())
        .fetch_optional(tx.as_mut())
        .await
        .map_err(IdentityError::from)?
        .map(|row| row.get("key"))
        .unwrap_or_default()
    };

    let staff = pb::IdentityStaffSummary {
        staff_id: row.staff_id,
        email: row.email.unwrap_or_default(),
        login_id: row.login_id.unwrap_or_default(),
        phone: row.phone.unwrap_or_default(),
        role_id: row.role_id,
        role_key,
        status: row.status,
        display_name: row.display_name.unwrap_or_default(),
    };

    let actor_id = req.actor.as_ref().and_then(|a| {
        if a.actor_id.is_empty() {
            None
        } else {
            Some(a.actor_id.clone())
        }
    });
    if actor_id.is_none() {
        return Err(IdentityError::permission_denied("actor_id is required"));
    }
    let actor_type = req
        .actor
        .as_ref()
        .and_then(|a| {
            if a.actor_type.is_empty() {
                None
            } else {
                Some(a.actor_type.clone())
            }
        })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(store_id.clone()),
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

    tx.commit().await.map_err(IdentityError::from)?;
    Ok(pb::IdentityUpdateStaffResponse {
        updated: true,
        staff: Some(staff),
    })
}

pub async fn invite_staff(
    state: &AppState,
    req: pb::IdentityInviteStaffRequest,
) -> IdentityResult<pb::IdentityInviteStaffResponse> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    require_owner(req.actor.as_ref())?;

    let invite_email = Email::parse(&req.email)?;
    if req.role_id.is_empty() {
        return Err(IdentityError::invalid_argument("role_id is required"));
    }

    let store_uuid = StoreId::parse(&store_id)?;
    let role_uuid = parse_uuid(&req.role_id, "role_id")?;
    let repo = PgIdentityRepository::new(&state.db);
    let role_row = repo.role_by_id(&store_uuid.as_uuid(), &role_uuid).await?;
    let Some(role_row) = role_row else {
        return Err(IdentityError::invalid_argument("role_id is invalid"));
    };
    let role_key = role_row.key;
    let role_name = role_row.name;
    if role_key == "owner" {
        return Err(IdentityError::invalid_argument(
            "owner role cannot be invited",
        ));
    }

    if repo
        .store_staff_exists_by_email(&store_uuid.as_uuid(), invite_email.as_str())
        .await?
    {
        return Err(IdentityError::already_exists("email already exists"));
    }

    if repo
        .invite_exists_by_email(&store_uuid.as_uuid(), invite_email.as_str())
        .await?
    {
        return Err(IdentityError::already_exists("invite already exists"));
    }

    let actor_id = req.actor.as_ref().and_then(|a| {
        if a.actor_id.is_empty() {
            None
        } else {
            Some(a.actor_id.clone())
        }
    });
    let created_by = match actor_id.as_ref() {
        Some(id) => Some(parse_uuid(id, "actor_id")?),
        None => None,
    };

    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;
    let staff_id = uuid::Uuid::new_v4();
    let invite_id = uuid::Uuid::new_v4();
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(7);

    repo.insert_staff_invite_tx(
        tx.as_mut(),
        &staff_id,
        &invite_id,
        &store_uuid.as_uuid(),
        invite_email.as_str(),
        &role_uuid,
        &token,
        created_by,
        expires_at,
        if req.display_name.is_empty() {
            None
        } else {
            Some(req.display_name.as_str())
        },
    )
    .await?;

    let store_name: String = repo
        .store_name(&store_uuid.as_uuid())
        .await?
        .unwrap_or_else(|| "Store".to_string());

    let email_config = email::EmailConfig::from_env();
    let _ = audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(store_id.clone()),
            actor_id,
            actor_type: req
                .actor
                .as_ref()
                .and_then(|a| {
                    if a.actor_type.is_empty() {
                        None
                    } else {
                        Some(a.actor_type.clone())
                    }
                })
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

    tx.commit().await.map_err(IdentityError::from)?;

    email::send_invite_email(
        &email_config,
        invite_email.as_str(),
        &store_name,
        Some(req.display_name.as_str()),
        role_name.as_deref(),
        &token,
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
) -> IdentityResult<pb::IdentityTransferOwnerResponse> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    require_owner(req.actor.as_ref())?;

    if req.new_owner_staff_id.is_empty() {
        return Err(IdentityError::invalid_argument(
            "new_owner_staff_id is required",
        ));
    }

    let store_uuid = StoreId::parse(&store_id)?;
    let new_owner_uuid = parse_uuid(&req.new_owner_staff_id, "new_owner_staff_id")?;
    let actor_id = req.actor.as_ref().and_then(|a| {
        if a.actor_id.is_empty() {
            None
        } else {
            Some(a.actor_id.clone())
        }
    });

    let repo = PgIdentityRepository::new(&state.db);
    let current_owner_id = repo.current_owner_id(&store_uuid.as_uuid()).await?;
    let Some(current_owner_id) = current_owner_id else {
        return Err(IdentityError::not_found("owner not found"));
    };
    if current_owner_id == req.new_owner_staff_id {
        return Err(IdentityError::invalid_argument(
            "new owner is already the owner",
        ));
    }

    if let Some(actor_id) = actor_id.as_ref() {
        if actor_id != &current_owner_id {
            return Err(IdentityError::permission_denied(
                "only owner can transfer ownership",
            ));
        }
    }

    let target_status = repo
        .staff_status(&store_uuid.as_uuid(), &new_owner_uuid)
        .await?;
    let Some(target_status) = target_status else {
        return Err(IdentityError::not_found("new owner staff not found"));
    };
    if target_status != "active" {
        return Err(IdentityError::invalid_argument("new owner must be active"));
    }

    let owner_role_id = repo
        .role_id_by_key(&store_uuid.as_uuid(), "owner")
        .await?
        .ok_or_else(|| IdentityError::not_found("owner role not found"))?;
    let staff_role_id = repo
        .role_id_by_key(&store_uuid.as_uuid(), "staff")
        .await?
        .ok_or_else(|| IdentityError::not_found("staff role not found"))?;

    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;
    repo.update_staff_role_tx(
        tx.as_mut(),
        &new_owner_uuid,
        &store_uuid.as_uuid(),
        &owner_role_id,
    )
    .await?;
    let current_owner_uuid = parse_uuid(&current_owner_id, "current_owner_id")?;
    repo.update_staff_role_tx(
        tx.as_mut(),
        &current_owner_uuid,
        &store_uuid.as_uuid(),
        &staff_role_id,
    )
    .await?;

    let _ = audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(store_id.clone()),
            actor_id,
            actor_type: req
                .actor
                .as_ref()
                .and_then(|a| {
                    if a.actor_type.is_empty() {
                        None
                    } else {
                        Some(a.actor_type.clone())
                    }
                })
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

    tx.commit().await.map_err(IdentityError::from)?;

    let new_owner =
        fetch_staff_summary(state, &store_uuid.as_uuid(), &req.new_owner_staff_id).await?;
    let previous_owner =
        fetch_staff_summary(state, &store_uuid.as_uuid(), &current_owner_id).await?;

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
    session_id: Option<String>,
) -> IdentityResult<pb::IdentitySignOutResponse> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    if let Some(session_id) = session_id {
        if let Ok(session_uuid) = uuid::Uuid::parse_str(&session_id) {
            if let Ok(store_uuid) = StoreId::parse(&store_id) {
                let _ = sqlx::query(
                    r#"
                    UPDATE store_staff_sessions
                    SET revoked_at = now()
                    WHERE id = $1 AND store_id = $2 AND revoked_at IS NULL
                    "#,
                )
                .bind(session_uuid)
                .bind(store_uuid.as_uuid())
                .execute(&state.db)
                .await;

                let _ = sqlx::query(
                    r#"
                    UPDATE store_staff_refresh_tokens
                    SET revoked_at = now()
                    WHERE session_id = $1 AND store_id = $2 AND revoked_at IS NULL
                    "#,
                )
                .bind(session_uuid)
                .bind(store_uuid.as_uuid())
                .execute(&state.db)
                .await;
            }
        }
    }
    let actor_id = actor.as_ref().and_then(|a| {
        if a.actor_id.is_empty() {
            None
        } else {
            Some(a.actor_id.clone())
        }
    });
    let actor_type = actor
        .as_ref()
        .and_then(|a| {
            if a.actor_type.is_empty() {
                None
            } else {
                Some(a.actor_type.clone())
            }
        })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record(
        state,
        audit::AuditInput {
            store_id: Some(store_id),
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

pub struct RefreshTokenResult {
    pub response: pb::IdentityRefreshTokenResponse,
    pub refresh_token: String,
}

pub async fn refresh_token(
    state: &AppState,
    req: pb::IdentityRefreshTokenRequest,
    refresh_token: String,
) -> IdentityResult<RefreshTokenResult> {
    if refresh_token.is_empty() {
        return Err(IdentityError::unauthenticated("refresh token is required"));
    }

    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let refresh_hash = hash_refresh_token(&refresh_token);
    let now = Utc::now();

    let row = sqlx::query(
        r#"
        SELECT id, staff_id, session_id, expires_at, revoked_at
        FROM store_staff_refresh_tokens
        WHERE token_hash = $1 AND store_id = $2
        "#,
    )
    .bind(refresh_hash)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(IdentityError::from)?;

    let Some(row) = row else {
        return Err(IdentityError::unauthenticated("refresh token not found"));
    };

    let token_id: uuid::Uuid = row.get("id");
    let staff_id: uuid::Uuid = row.get("staff_id");
    let session_id: uuid::Uuid = row.get("session_id");
    let revoked_at: Option<chrono::DateTime<Utc>> = row.get("revoked_at");
    let expires_at: chrono::DateTime<Utc> = row.get("expires_at");

    if revoked_at.is_some() || expires_at <= now {
        return Err(IdentityError::unauthenticated("refresh token expired"));
    }

    let staff_row = sqlx::query(
        r#"
        SELECT s.id::text as staff_id, s.status, r.key as role_key
        FROM store_staff s
        LEFT JOIN store_roles r ON r.id = s.role_id
        WHERE s.id = $1 AND s.store_id = $2
        "#,
    )
    .bind(staff_id)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(IdentityError::from)?;

    let Some(staff_row) = staff_row else {
        return Err(IdentityError::unauthenticated("staff not found"));
    };

    let status: String = staff_row.get("status");
    if status != "active" {
        return Err(IdentityError::unauthenticated("staff is inactive"));
    }

    let role_key: String = staff_row
        .get::<Option<String>, _>("role_key")
        .unwrap_or_default();
    let staff_id_str: String = staff_row.get("staff_id");

    let jwt_secret = std::env::var("AUTH_JWT_SECRET")
        .map_err(|_| IdentityError::internal("AUTH_JWT_SECRET is required"))?;

    let exp = now + Duration::minutes(ACCESS_TOKEN_TTL_MINUTES);
    let claims = JwtClaims {
        sub: staff_id_str.clone(),
        actor_type: role_key.clone(),
        tenant_id: tenant_id.clone(),
        store_id: store_id.clone(),
        jti: session_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| IdentityError::internal("failed to sign token"))?;

    let new_refresh_token = uuid::Uuid::new_v4().to_string();
    let new_refresh_id = uuid::Uuid::new_v4();
    let new_refresh_hash = hash_refresh_token(&new_refresh_token);
    let refresh_expires_at = now + Duration::days(REFRESH_TOKEN_TTL_DAYS);

    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;

    sqlx::query(
        r#"
        UPDATE store_staff_refresh_tokens
        SET revoked_at = now(), replaced_by = $1, last_used_at = now()
        WHERE id = $2 AND revoked_at IS NULL
        "#,
    )
    .bind(new_refresh_id)
    .bind(token_id)
    .execute(tx.as_mut())
    .await
    .map_err(IdentityError::from)?;

    sqlx::query(
        r#"
        INSERT INTO store_staff_refresh_tokens
            (id, store_id, staff_id, session_id, token_hash, expires_at)
        VALUES ($1,$2,$3,$4,$5,$6)
        "#,
    )
    .bind(new_refresh_id)
    .bind(store_uuid.as_uuid())
    .bind(staff_id)
    .bind(session_id)
    .bind(new_refresh_hash)
    .bind(refresh_expires_at)
    .execute(tx.as_mut())
    .await
    .map_err(IdentityError::from)?;

    sqlx::query(
        r#"
        UPDATE store_staff_sessions
        SET last_seen_at = now(), expires_at = $2
        WHERE id = $1
        "#,
    )
    .bind(session_id)
    .bind(exp)
    .execute(tx.as_mut())
    .await
    .map_err(IdentityError::from)?;

    tx.commit().await.map_err(IdentityError::from)?;

    Ok(RefreshTokenResult {
        response: pb::IdentityRefreshTokenResponse {
            access_token: token,
            store_id,
            tenant_id,
            staff_id: staff_id_str,
            role: role_key,
            expires_at: chrono_to_timestamp(Some(exp)),
        },
        refresh_token: new_refresh_token,
    })
}

pub async fn create_staff(
    state: &AppState,
    req: pb::IdentityCreateStaffRequest,
) -> IdentityResult<pb::IdentityCreateStaffResponse> {
    let store = req.store.clone();
    let tenant = req.tenant.clone();
    let email = req.email.clone();
    let login_id = req.login_id.clone();
    let phone = req.phone.clone();
    let role_id = req.role_id.clone();
    let display_name = req.display_name.clone();
    let actor = req.actor.clone();

    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;
    let resp = create_staff_core(
        state,
        &mut tx,
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

    let actor_id = actor.as_ref().and_then(|a| {
        if a.actor_id.is_empty() {
            None
        } else {
            Some(a.actor_id.clone())
        }
    });
    let actor_type = actor
        .as_ref()
        .and_then(|a| {
            if a.actor_type.is_empty() {
                None
            } else {
                Some(a.actor_type.clone())
            }
        })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(resp.store_id.clone()),
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

    tx.commit().await.map_err(IdentityError::from)?;
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
) -> IdentityResult<pb::IdentityCreateRoleResponse> {
    let (store_id, _tenant_id) =
        resolve_store_context(state, req.store.clone(), req.tenant.clone()).await?;
    let store = req.store.clone();
    let tenant = req.tenant.clone();
    let key = req.key.clone();
    let name = req.name.clone();
    let description = req.description.clone();
    let permissions = req.permission_keys.clone();
    let actor = req.actor.clone();

    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;
    let resp = create_role_core(
        state,
        &mut tx,
        store,
        tenant,
        key.clone(),
        name.clone(),
        description.clone(),
        permissions,
    )
    .await?;

    if let Some(role) = resp.role.as_ref() {
        let actor_id = actor.as_ref().and_then(|a| {
            if a.actor_id.is_empty() {
                None
            } else {
                Some(a.actor_id.clone())
            }
        });
        let actor_type = actor
            .as_ref()
            .and_then(|a| {
                if a.actor_type.is_empty() {
                    None
                } else {
                    Some(a.actor_type.clone())
                }
            })
            .unwrap_or_else(|| "staff".to_string());
        let tenant_id = resolve_store_context(state, req.store, req.tenant)
            .await
            .map(|(_, tenant_id)| tenant_id)?;

        let _ = audit::record_tx(
            &mut tx,
            audit::AuditInput {
                store_id: Some(store_id.clone()),
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

    tx.commit().await.map_err(IdentityError::from)?;
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
) -> IdentityResult<pb::IdentityAssignRoleResponse> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    if req.staff_id.is_empty() || req.role_id.is_empty() {
        return Err(IdentityError::invalid_argument(
            "staff_id and role_id are required",
        ));
    }

    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgIdentityRepository::new(&state.db);
    let role_store_id = repo
        .role_store_id(&req.role_id)
        .await?
        .ok_or_else(|| IdentityError::invalid_argument("role_id is invalid"))?;
    if role_store_id != store_id {
        return Err(IdentityError::permission_denied(
            "role does not belong to store",
        ));
    }

    let staff_uuid = parse_uuid(&req.staff_id, "staff_id")?;
    let current_role = repo
        .staff_role_key(&store_uuid.as_uuid(), &staff_uuid)
        .await?;
    if let Some(current_role) = current_role {
        if current_role == "owner" {
            return Err(IdentityError::permission_denied(
                "owner role cannot be assigned",
            ));
        }
    }

    let role_uuid = parse_uuid(&req.role_id, "role_id")?;
    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;
    repo.update_staff_role_tx(tx.as_mut(), &staff_uuid, &store_uuid.as_uuid(), &role_uuid)
        .await?;

    let actor_id = req.actor.as_ref().and_then(|a| {
        if a.actor_id.is_empty() {
            None
        } else {
            Some(a.actor_id.clone())
        }
    });
    let actor_type = req
        .actor
        .as_ref()
        .and_then(|a| {
            if a.actor_type.is_empty() {
                None
            } else {
                Some(a.actor_type.clone())
            }
        })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(store_id.clone()),
            actor_id,
            actor_type,
            action: IdentityAuditAction::RoleAssign.into(),
            target_type: Some("store_staff".to_string()),
            target_id: Some(req.staff_id.clone()),
            request_id: None,
            ip_address: None,
            user_agent: None,
            before_json: None,
            after_json: None,
            metadata_json: Some(serde_json::json!({
                "role_id": req.role_id,
            })),
        },
    )
    .await?;

    tx.commit().await.map_err(IdentityError::from)?;

    Ok(pb::IdentityAssignRoleResponse { assigned: true })
}

pub async fn list_roles(
    state: &AppState,
    req: pb::IdentityListRolesRequest,
) -> IdentityResult<pb::IdentityListRolesResponse> {
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
) -> IdentityResult<pb::IdentityListRolesWithPermissionsResponse> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgIdentityRepository::new(&state.db);
    let role_rows = repo.list_roles(&store_uuid.as_uuid()).await?;

    let mut roles: Vec<pb::IdentityRoleDetail> = role_rows
        .iter()
        .map(|row| pb::IdentityRoleDetail {
            id: row.id.clone(),
            key: row.key.clone(),
            name: row.name.clone(),
            description: row.description.clone().unwrap_or_default(),
            permission_keys: Vec::new(),
        })
        .collect();

    if !roles.is_empty() {
        let role_ids: Vec<uuid::Uuid> = role_rows
            .iter()
            .filter_map(|row| uuid::Uuid::parse_str(&row.id).ok())
            .collect();
        let permission_rows = repo.list_role_permissions(&role_ids).await?;
        for (role_id, key) in permission_rows {
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
) -> IdentityResult<pb::IdentityUpdateRoleResponse> {
    if req.role_id.is_empty() {
        return Err(IdentityError::invalid_argument("role_id is required"));
    }

    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let role_uuid = parse_uuid(&req.role_id, "role_id")?;
    let permission_keys = req.permission_keys.clone();

    let repo = PgIdentityRepository::new(&state.db);
    let rows = repo.permissions_by_keys(&permission_keys).await?;

    if rows.len() != permission_keys.len() && !permission_keys.is_empty() {
        return Err(IdentityError::invalid_argument(
            "unknown permission key included",
        ));
    }

    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;
    let repo = PgIdentityRepository::new(&state.db);
    let updated = repo
        .update_role_tx(
            tx.as_mut(),
            &store_uuid.as_uuid(),
            &role_uuid,
            &req.name,
            &req.description,
        )
        .await?;

    let Some(row) = updated else {
        return Ok(pb::IdentityUpdateRoleResponse {
            updated: false,
            role: None,
        });
    };

    repo.delete_role_permissions_tx(tx.as_mut(), &role_uuid)
        .await?;

    for (permission_id, _key) in rows {
        let permission_uuid = parse_uuid(&permission_id, "permission_id")?;
        repo.insert_role_permission_tx(tx.as_mut(), &role_uuid, &permission_uuid)
            .await?;
    }

    let role = pb::IdentityRoleDetail {
        id: row.id,
        key: row.key,
        name: row.name,
        description: row.description.unwrap_or_default(),
        permission_keys,
    };

    let actor_id = req.actor.as_ref().and_then(|a| {
        if a.actor_id.is_empty() {
            None
        } else {
            Some(a.actor_id.clone())
        }
    });
    let actor_type = req
        .actor
        .as_ref()
        .and_then(|a| {
            if a.actor_type.is_empty() {
                None
            } else {
                Some(a.actor_type.clone())
            }
        })
        .unwrap_or_else(|| "staff".to_string());

    let _ = audit::record_tx(
        &mut tx,
        audit::AuditInput {
            store_id: Some(store_id.clone()),
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

    tx.commit().await.map_err(IdentityError::from)?;
    Ok(pb::IdentityUpdateRoleResponse {
        updated: true,
        role: Some(role),
    })
}

pub async fn delete_role(
    state: &AppState,
    req: pb::IdentityDeleteRoleRequest,
) -> IdentityResult<pb::IdentityDeleteRoleResponse> {
    if req.role_id.is_empty() {
        return Err(IdentityError::invalid_argument("role_id is required"));
    }

    let (store_id, tenant_id) = resolve_store_context(state, req.store, req.tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let role_uuid = parse_uuid(&req.role_id, "role_id")?;

    let repo = PgIdentityRepository::new(&state.db);
    if repo.role_attached(&role_uuid).await? {
        return Err(IdentityError::failed_precondition(
            "role is attached to staff",
        ));
    }

    let mut tx = state.db.begin().await.map_err(IdentityError::from)?;
    let deleted = repo
        .delete_role_tx(tx.as_mut(), &store_uuid.as_uuid(), &role_uuid)
        .await?;

    if let Some(row) = deleted {
        let actor_id = req.actor.as_ref().and_then(|a| {
            if a.actor_id.is_empty() {
                None
            } else {
                Some(a.actor_id.clone())
            }
        });
        let actor_type = req
            .actor
            .as_ref()
            .and_then(|a| {
                if a.actor_type.is_empty() {
                    None
                } else {
                    Some(a.actor_type.clone())
                }
            })
            .unwrap_or_else(|| "staff".to_string());

        let _ = audit::record_tx(
            &mut tx,
            audit::AuditInput {
                store_id: Some(store_id.clone()),
                actor_id,
                actor_type,
                action: IdentityAuditAction::RoleDelete.into(),
                target_type: Some("store_role".to_string()),
                target_id: Some(row.id.clone()),
                request_id: None,
                ip_address: None,
                user_agent: None,
                before_json: Some(serde_json::json!({
                    "role_id": row.id,
                    "key": row.key,
                    "name": row.name.unwrap_or_default(),
                })),
                after_json: None,
                metadata_json: None,
            },
        )
        .await?;

        tx.commit().await.map_err(IdentityError::from)?;
        return Ok(pb::IdentityDeleteRoleResponse { deleted: true });
    }

    tx.commit().await.map_err(IdentityError::from)?;
    Ok(pb::IdentityDeleteRoleResponse { deleted: false })
}

struct SignInCoreResult {
    access_token: String,
    store_id: String,
    tenant_id: String,
    staff_id: String,
    role: String,
    expires_at: Option<pbjson_types::Timestamp>,
    refresh_token: String,
}

async fn sign_in_core(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    email: String,
    login_id: String,
    phone: String,
    password: String,
) -> IdentityResult<SignInCoreResult> {
    if password.is_empty() {
        return Err(IdentityError::invalid_argument("password is required"));
    }

    let email = Email::parse_optional(&email)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    let phone = Phone::parse_optional(&phone)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();

    let (store_id, tenant_id) =
        resolve_store_context_without_token_guard(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let _tenant_uuid = TenantId::parse(&tenant_id)?;

    let repo = PgIdentityRepository::new(&state.db);
    let row = if !email.is_empty() {
        repo.fetch_active_staff_by_email(&store_uuid.as_uuid(), &email)
            .await?
    } else if !login_id.is_empty() {
        repo.fetch_active_staff_by_login_id(&store_uuid.as_uuid(), &login_id)
            .await?
    } else if !phone.is_empty() {
        repo.fetch_active_staff_by_phone(&store_uuid.as_uuid(), &phone)
            .await?
    } else {
        return Err(IdentityError::invalid_argument(
            "email or login_id or phone is required",
        ));
    };

    let Some(row) = row else {
        return Err(IdentityError::unauthenticated("invalid credentials"));
    };

    let hash = row
        .password_hash
        .ok_or_else(|| IdentityError::unauthenticated("invalid credentials"))?;

    let parsed_hash = PasswordHash::new(&hash)
        .map_err(|_| IdentityError::unauthenticated("invalid credentials"))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| IdentityError::unauthenticated("invalid credentials"))?;

    let staff_id: String = row.staff_id;
    let role: String = row.role_key;
    let session_id = uuid::Uuid::new_v4();

    let jwt_secret = std::env::var("AUTH_JWT_SECRET")
        .map_err(|_| IdentityError::internal("AUTH_JWT_SECRET is required"))?;

    let now = chrono::Utc::now();
    let exp = now + chrono::Duration::minutes(ACCESS_TOKEN_TTL_MINUTES);
    let claims = JwtClaims {
        sub: staff_id.clone(),
        actor_type: role.clone(),
        tenant_id: tenant_id.clone(),
        store_id: store_id.clone(),
        jti: session_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| IdentityError::internal("failed to sign token"))?;

    let ctx = crate::rpc::request_context::current();
    let ip_address = ctx.as_ref().and_then(|c| c.ip_address.clone());
    let user_agent = ctx.as_ref().and_then(|c| c.user_agent.clone());
    let refresh_expires_at = now + chrono::Duration::days(REFRESH_TOKEN_TTL_DAYS);
    let refresh_token = uuid::Uuid::new_v4().to_string();
    let refresh_token_id = uuid::Uuid::new_v4();
    let refresh_hash = hash_refresh_token(&refresh_token);

    sqlx::query(
        r#"
        INSERT INTO store_staff_sessions
            (id, store_id, staff_id, ip_address, user_agent, last_seen_at, expires_at)
        VALUES ($1,$2,$3,$4,$5,now(),$6)
        "#,
    )
    .bind(session_id)
    .bind(store_uuid.as_uuid())
    .bind(
        parse_uuid(&staff_id, "staff_id")
            .map_err(|_| IdentityError::internal("invalid staff_id"))?,
    )
    .bind(ip_address)
    .bind(user_agent)
    .bind(exp)
    .execute(&state.db)
    .await
    .map_err(|_| IdentityError::internal("failed to create staff session"))?;

    sqlx::query(
        r#"
        INSERT INTO store_staff_refresh_tokens
            (id, store_id, staff_id, session_id, token_hash, expires_at)
        VALUES ($1,$2,$3,$4,$5,$6)
        "#,
    )
    .bind(refresh_token_id)
    .bind(store_uuid.as_uuid())
    .bind(
        parse_uuid(&staff_id, "staff_id")
            .map_err(|_| IdentityError::internal("invalid staff_id"))?,
    )
    .bind(session_id)
    .bind(refresh_hash)
    .bind(refresh_expires_at)
    .execute(&state.db)
    .await
    .map_err(|_| IdentityError::internal("failed to create refresh token"))?;

    Ok(SignInCoreResult {
        access_token: token,
        store_id,
        tenant_id,
        staff_id,
        role,
        expires_at: chrono_to_timestamp(Some(exp)),
        refresh_token,
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
    tx: &mut Transaction<'_, Postgres>,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    email: String,
    login_id: String,
    phone: String,
    password: String,
    role_id: String,
    display_name: String,
) -> IdentityResult<CreateStaffCoreResult> {
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
        return Err(IdentityError::invalid_argument("role_id is required"));
    }
    if email.is_empty() && login_id.is_empty() && phone.is_empty() {
        return Err(IdentityError::invalid_argument(
            "email or login_id or phone is required",
        ));
    }
    if password.is_empty() {
        return Err(IdentityError::invalid_argument("password is required"));
    }

    let password_hash = hash_password(&password)?;
    let staff_id = uuid::Uuid::new_v4();

    let role_uuid = parse_uuid(&role_id, "role_id")?;
    let repo = PgIdentityRepository::new(&state.db);
    let role_row = repo.role_by_id(&store_uuid.as_uuid(), &role_uuid).await?;
    let role_key = role_row
        .map(|row| row.key)
        .ok_or_else(|| IdentityError::invalid_argument("role_id is invalid"))?;
    if role_key == "owner" {
        return Err(IdentityError::invalid_argument(
            "owner role cannot be assigned",
        ));
    }

    let email_value = if email.is_empty() {
        None
    } else {
        Some(email.as_str())
    };
    let login_id_value = if login_id.is_empty() {
        None
    } else {
        Some(login_id.as_str())
    };
    let phone_value = if phone.is_empty() {
        None
    } else {
        Some(phone.as_str())
    };
    let display_name_value = if display_name.is_empty() {
        None
    } else {
        Some(display_name.as_str())
    };
    repo.insert_staff_tx(
        tx.as_mut(),
        &staff_id,
        &store_uuid.as_uuid(),
        email_value,
        login_id_value,
        phone_value,
        &password_hash,
        &role_uuid,
        "active",
        display_name_value,
    )
    .await?;

    Ok(CreateStaffCoreResult {
        staff_id: staff_id.to_string(),
        store_id,
        tenant_id,
        role_id,
        role_key,
        display_name,
    })
}

fn require_owner(actor: Option<&pb::ActorContext>) -> IdentityResult<()> {
    let actor_type = actor
        .and_then(|a| {
            if a.actor_type.is_empty() {
                None
            } else {
                Some(a.actor_type.as_str())
            }
        })
        .unwrap_or("");
    if actor_type != "owner" {
        return Err(IdentityError::permission_denied("owner role is required"));
    }
    Ok(())
}

async fn fetch_staff_summary(
    state: &AppState,
    store_uuid: &uuid::Uuid,
    staff_id: &str,
) -> IdentityResult<pb::IdentityStaffSummary> {
    let staff_uuid = parse_uuid(staff_id, "staff_id")?;
    let repo = PgIdentityRepository::new(&state.db);
    let row = repo
        .staff_summary(store_uuid, &staff_uuid)
        .await?
        .ok_or_else(|| IdentityError::not_found("staff not found"))?;

    Ok(pb::IdentityStaffSummary {
        staff_id: row.staff_id,
        email: row.email.unwrap_or_default(),
        login_id: row.login_id.unwrap_or_default(),
        phone: row.phone.unwrap_or_default(),
        role_id: row.role_id,
        role_key: row.role_key,
        status: row.status,
        display_name: row.display_name.unwrap_or_default(),
    })
}

fn hash_password(password: &str) -> IdentityResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| IdentityError::internal("failed to hash password"))
}

fn hash_refresh_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    hex::encode(hash)
}

#[derive(Serialize)]
struct JwtClaims {
    sub: String,
    actor_type: String,
    tenant_id: String,
    store_id: String,
    jti: String,
    exp: usize,
    iat: usize,
}

async fn create_role_core(
    state: &AppState,
    tx: &mut Transaction<'_, Postgres>,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    key: String,
    name: String,
    description: String,
    permission_keys: Vec<String>,
) -> IdentityResult<pb::CreateRoleResponse> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;

    if key.is_empty() || name.is_empty() {
        return Err(IdentityError::invalid_argument("key and name are required"));
    }

    let role_id = uuid::Uuid::new_v4();
    let repo = PgIdentityRepository::new(&state.db);
    repo.insert_role_tx(
        tx.as_mut(),
        &store_uuid.as_uuid(),
        &role_id,
        &key,
        &name,
        &description,
    )
    .await?;

    if !permission_keys.is_empty() {
        let rows = repo.permissions_by_keys(&permission_keys).await?;
        for (permission_id, _key) in rows {
            let permission_uuid = parse_uuid(&permission_id, "permission_id")?;
            repo.insert_role_permission_tx(tx.as_mut(), &role_id, &permission_uuid)
                .await?;
        }
    }

    Ok(pb::CreateRoleResponse {
        role: Some(pb::Role {
            id: role_id.to_string(),
            key,
            name,
            description,
        }),
    })
}

async fn list_roles_core(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> IdentityResult<pb::ListRolesResponse> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let repo = PgIdentityRepository::new(&state.db);
    let rows = repo.list_roles(&store_uuid.as_uuid()).await?;

    let roles = rows
        .into_iter()
        .map(|row| pb::Role {
            id: row.id,
            key: row.key,
            name: row.name,
            description: row.description.unwrap_or_default(),
        })
        .collect();

    Ok(pb::ListRolesResponse { roles })
}
