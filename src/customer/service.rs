use chrono::Utc;
use sqlx::Row;

use crate::{
    AppState,
    pb::pb,
    infrastructure::{audit, outbox},
    customer::error::{CustomerError, CustomerResult},
    shared::validation::{Email, Phone},
    shared::{
        ids::{parse_uuid, StoreId, TenantId},
        audit_helpers::{audit_input, to_json_opt},
        time::chrono_to_timestamp,
        audit_action::CustomerAuditAction,
    },
};

const DEFAULT_PROFILE_STATUS: &str = "active";
const DEFAULT_CUSTOMER_STATUS: &str = "active";

pub async fn list_customers(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    query: String,
) -> CustomerResult<Vec<pb::CustomerSummary>> {
    let store_uuid = StoreId::parse(&store_id).map_err(CustomerError::from)?;
    let tenant_uuid = TenantId::parse(&tenant_id).map_err(CustomerError::from)?;
    let q = query.trim();

    let rows = if q.is_empty() {
        sqlx::query(
            r#"
            SELECT c.id::text as customer_id,
                   cp.id::text as profile_id,
                   cp.store_id::text as store_id,
                   cp.name, cp.email, cp.phone, cp.status,
                   c.created_at
            FROM customers c
            JOIN customer_profiles cp ON cp.customer_id = c.id
            WHERE c.tenant_id = $1
              AND cp.store_id = $2
            ORDER BY c.created_at DESC
            LIMIT 200
            "#,
        )
        .bind(tenant_uuid.as_uuid())
        .bind(store_uuid.as_uuid())
        .fetch_all(&state.db)
        .await
        .map_err(CustomerError::from)?
    } else {
        let pattern = format!("%{}%", q);
        sqlx::query(
            r#"
            SELECT c.id::text as customer_id,
                   cp.id::text as profile_id,
                   cp.store_id::text as store_id,
                   cp.name, cp.email, cp.phone, cp.status,
                   c.created_at
            FROM customers c
            JOIN customer_profiles cp ON cp.customer_id = c.id
            WHERE c.tenant_id = $1
              AND cp.store_id = $2
              AND (cp.name ILIKE $3 OR cp.email ILIKE $3 OR cp.phone ILIKE $3)
            ORDER BY c.created_at DESC
            LIMIT 200
            "#,
        )
        .bind(tenant_uuid.as_uuid())
        .bind(store_uuid.as_uuid())
        .bind(pattern)
        .fetch_all(&state.db)
        .await
        .map_err(CustomerError::from)?
    };

    let customers = rows
        .into_iter()
        .map(|row| pb::CustomerSummary {
            customer_id: row.get("customer_id"),
            profile_id: row.get("profile_id"),
            store_id: row.get("store_id"),
            name: row.get::<String, _>("name"),
            email: row.get::<Option<String>, _>("email").unwrap_or_default(),
            phone: row.get::<Option<String>, _>("phone").unwrap_or_default(),
            status: row.get("status"),
            created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        })
        .collect();

    Ok(customers)
}

pub async fn get_customer(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    customer_id: String,
) -> CustomerResult<(pb::Customer, pb::CustomerProfile, Vec<pb::CustomerIdentity>, Vec<pb::CustomerAddress>)> {
    let store_uuid = StoreId::parse(&store_id).map_err(CustomerError::from)?;
    let tenant_uuid = TenantId::parse(&tenant_id).map_err(CustomerError::from)?;
    let customer_uuid = parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?;

    let row = sqlx::query(
        r#"
        SELECT id::text as id, tenant_id::text as tenant_id, status, created_at, updated_at
        FROM customers
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(customer_uuid)
    .bind(tenant_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(CustomerError::from)?;

    let Some(row) = row else {
        return Err(CustomerError::NotFound("customer not found".to_string()));
    };

    let customer = pb::Customer {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        status: row.get("status"),
        created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
    };

    let profile_row = sqlx::query(
        r#"
        SELECT id::text as id, customer_id::text as customer_id, store_id::text as store_id,
               name, email, phone, status, notes, created_at, updated_at
        FROM customer_profiles
        WHERE customer_id = $1 AND store_id = $2
        "#,
    )
    .bind(customer_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(CustomerError::from)?;

    let profile = if let Some(row) = profile_row {
        pb::CustomerProfile {
            id: row.get("id"),
            customer_id: row.get("customer_id"),
            store_id: row.get("store_id"),
            name: row.get::<String, _>("name"),
            email: row.get::<Option<String>, _>("email").unwrap_or_default(),
            phone: row.get::<Option<String>, _>("phone").unwrap_or_default(),
            status: row.get("status"),
            notes: row.get::<Option<String>, _>("notes").unwrap_or_default(),
            created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
            updated_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
        }
    } else {
        pb::CustomerProfile {
            id: String::new(),
            customer_id: customer_id.clone(),
            store_id: store_id.clone(),
            name: String::new(),
            email: String::new(),
            phone: String::new(),
            status: DEFAULT_PROFILE_STATUS.to_string(),
            notes: String::new(),
            created_at: None,
            updated_at: None,
        }
    };

    let identity_rows = sqlx::query(
        r#"
        SELECT id::text as id, customer_id::text as customer_id, tenant_id::text as tenant_id,
               identity_type, identity_value, verified, source, created_at
        FROM customer_identities
        WHERE customer_id = $1 AND tenant_id = $2
        ORDER BY created_at ASC
        "#,
    )
    .bind(customer_uuid)
    .bind(tenant_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(CustomerError::from)?;

    let identities = identity_rows
        .into_iter()
        .map(|row| pb::CustomerIdentity {
            id: row.get("id"),
            customer_id: row.get("customer_id"),
            tenant_id: row.get("tenant_id"),
            identity_type: row.get("identity_type"),
            identity_value: row.get("identity_value"),
            verified: row.get("verified"),
            source: row.get("source"),
            created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        })
        .collect();

    let address_rows = sqlx::query(
        r#"
        SELECT id::text as id, customer_id::text as customer_id, type, name,
               postal_code, prefecture, city, line1, line2, phone,
               created_at, updated_at
        FROM customer_addresses
        WHERE customer_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(customer_uuid)
    .fetch_all(&state.db)
    .await
    .map_err(CustomerError::from)?;

    let addresses = address_rows
        .into_iter()
        .map(|row| pb::CustomerAddress {
            id: row.get("id"),
            customer_id: row.get("customer_id"),
            r#type: row.get("type"),
            name: row.get("name"),
            postal_code: row.get("postal_code"),
            prefecture: row.get("prefecture"),
            city: row.get("city"),
            line1: row.get("line1"),
            line2: row.get::<Option<String>, _>("line2").unwrap_or_default(),
            phone: row.get::<Option<String>, _>("phone").unwrap_or_default(),
            created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
            updated_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
        })
        .collect();

    Ok((customer, profile, identities, addresses))
}

pub async fn create_customer(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    mut profile: pb::CustomerProfileInput,
    identities: Vec<pb::CustomerIdentityInput>,
    actor: Option<pb::ActorContext>,
) -> CustomerResult<(pb::Customer, pb::CustomerProfile, bool)> {
    let store_uuid = StoreId::parse(&store_id).map_err(CustomerError::from)?;
    let tenant_uuid = TenantId::parse(&tenant_id).map_err(CustomerError::from)?;
    let normalized_email = Email::parse_optional(&profile.email)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    let normalized_phone = Phone::parse_optional(&profile.phone)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    profile.email = normalized_email;
    profile.phone = normalized_phone;
    validate_customer_profile(&profile, &identities)?;

    let mut identity_inputs = normalize_identities(&profile, identities);
    for identity in &mut identity_inputs {
        identity.identity_value = normalize_identity(&identity.identity_type, &identity.identity_value);
    }
    let mut matched_customer_id: Option<String> = None;

    if !identity_inputs.is_empty() {
        let mut best_match: Option<(String, bool, chrono::DateTime<Utc>)> = None;
        for identity in &identity_inputs {
            let row = sqlx::query(
                r#"
                SELECT customer_id::text as customer_id, verified, created_at
                FROM customer_identities
                WHERE tenant_id = $1 AND identity_type = $2 AND identity_value = $3
                ORDER BY verified DESC, created_at ASC
                LIMIT 1
                "#,
            )
            .bind(tenant_uuid.as_uuid())
            .bind(&identity.identity_type)
            .bind(&identity.identity_value)
            .fetch_optional(&state.db)
            .await
            .map_err(CustomerError::from)?;
            if let Some(row) = row {
                let candidate = (
                    row.get::<String, _>("customer_id"),
                    row.get::<bool, _>("verified"),
                    row.get::<chrono::DateTime<Utc>, _>("created_at"),
                );
                best_match = match best_match {
                    None => Some(candidate),
                    Some(current) => {
                        if candidate.1 && !current.1 {
                            Some(candidate)
                        } else if candidate.1 == current.1 && candidate.2 < current.2 {
                            Some(candidate)
                        } else {
                            Some(current)
                        }
                    }
                };
            }
        }
        if let Some((customer_id, _, _)) = best_match {
            matched_customer_id = Some(customer_id);
        }
    }

    let (customer_id, matched_existing) = if let Some(customer_id) = matched_customer_id {
        (customer_id, true)
    } else {
        (uuid::Uuid::new_v4().to_string(), false)
    };

    if !matched_existing {
        sqlx::query(
            r#"
            INSERT INTO customers (id, tenant_id, status)
            VALUES ($1,$2,$3)
            "#,
        )
        .bind(parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?)
        .bind(tenant_uuid.as_uuid())
        .bind(DEFAULT_CUSTOMER_STATUS)
        .execute(&state.db)
        .await
        .map_err(CustomerError::from)?;
    }

    let profile_row = sqlx::query(
        r#"
        INSERT INTO customer_profiles (id, customer_id, store_id, name, email, phone, status, notes)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ON CONFLICT (customer_id, store_id)
        DO UPDATE SET name = EXCLUDED.name,
                      email = EXCLUDED.email,
                      phone = EXCLUDED.phone,
                      status = EXCLUDED.status,
                      notes = EXCLUDED.notes,
                      updated_at = now()
        RETURNING id::text as id, customer_id::text as customer_id, store_id::text as store_id,
                  name, email, phone, status, notes, created_at, updated_at
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?)
    .bind(store_uuid.as_uuid())
    .bind(profile.name)
    .bind(if profile.email.is_empty() { None } else { Some(profile.email) })
    .bind(if profile.phone.is_empty() { None } else { Some(profile.phone) })
    .bind(if profile.status.is_empty() { DEFAULT_PROFILE_STATUS } else { profile.status.as_str() })
    .bind(if profile.notes.is_empty() { None } else { Some(profile.notes) })
    .fetch_one(&state.db)
    .await
    .map_err(CustomerError::from)?;

    for identity in &identity_inputs {
        sqlx::query(
            r#"
            INSERT INTO customer_identities
                (id, tenant_id, customer_id, identity_type, identity_value, verified, source)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            ON CONFLICT (tenant_id, identity_type, identity_value) DO NOTHING
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(tenant_uuid.as_uuid())
        .bind(parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?)
        .bind(&identity.identity_type)
        .bind(&identity.identity_value)
        .bind(identity.verified)
        .bind("admin")
        .execute(&state.db)
        .await
        .map_err(CustomerError::from)?;
    }

    let customer = pb::Customer {
        id: customer_id.clone(),
        tenant_id,
        status: DEFAULT_CUSTOMER_STATUS.to_string(),
        created_at: None,
        updated_at: None,
    };
    let profile = pb::CustomerProfile {
        id: profile_row.get("id"),
        customer_id: profile_row.get("customer_id"),
        store_id: profile_row.get("store_id"),
        name: profile_row.get("name"),
        email: profile_row.get::<Option<String>, _>("email").unwrap_or_default(),
        phone: profile_row.get::<Option<String>, _>("phone").unwrap_or_default(),
        status: profile_row.get("status"),
        notes: profile_row.get::<Option<String>, _>("notes").unwrap_or_default(),
        created_at: chrono_to_timestamp(Some(profile_row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(profile_row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
    };

    let _ = audit::record(
        state,
        audit_input(
            customer.tenant_id.clone(),
            CustomerAuditAction::Create.into(),
            Some("customer"),
            Some(customer.id.clone()),
            None,
            to_json_opt(Some(profile.clone())),
            actor,
        ),
    )
    .await?;

    let _ = outbox::enqueue(
        state,
        outbox::OutboxEventInput {
            tenant_id: customer.tenant_id.clone(),
            store_id: Some(store_id.clone()),
            aggregate_type: "customer".to_string(),
            aggregate_id: customer.id.clone(),
            event_type: "customer.profile_upsert".to_string(),
            payload_json: serde_json::json!({
                "tenant_id": customer.tenant_id,
                "source_store_id": store_id,
                "customer_id": customer.id,
                "profile": {
                    "name": profile.name,
                    "email": profile.email,
                    "phone": profile.phone,
                    "status": profile.status,
                    "notes": profile.notes,
                }
            }),
        },
    )
    .await?;

    for identity in identity_inputs {
        let _ = outbox::enqueue(
            state,
            outbox::OutboxEventInput {
                tenant_id: customer.tenant_id.clone(),
                store_id: Some(store_id.clone()),
                aggregate_type: "customer".to_string(),
                aggregate_id: customer.id.clone(),
                event_type: "customer.identity_upsert".to_string(),
                payload_json: serde_json::json!({
                    "tenant_id": customer.tenant_id,
                    "source_store_id": store_id,
                    "customer_id": customer.id,
                    "identity": {
                        "identity_type": identity.identity_type,
                        "identity_value": identity.identity_value,
                        "verified": identity.verified,
                        "source": "admin"
                    }
                }),
            },
        )
        .await?;
    }

    Ok((customer, profile, matched_existing))
}

pub async fn update_customer(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    customer_id: String,
    mut profile: pb::CustomerProfileInput,
    customer_status: String,
    actor: Option<pb::ActorContext>,
) -> CustomerResult<(pb::Customer, pb::CustomerProfile)> {
    let store_uuid = StoreId::parse(&store_id).map_err(CustomerError::from)?;
    let tenant_uuid = TenantId::parse(&tenant_id).map_err(CustomerError::from)?;
    let customer_uuid = parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?;
    let normalized_email = Email::parse_optional(&profile.email)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    let normalized_phone = Phone::parse_optional(&profile.phone)?
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    profile.email = normalized_email;
    profile.phone = normalized_phone;

    if !customer_status.is_empty() {
        sqlx::query(
            r#"
            UPDATE customers
            SET status = $1, updated_at = now()
            WHERE id = $2 AND tenant_id = $3
            "#,
        )
        .bind(&customer_status)
        .bind(customer_uuid)
        .bind(tenant_uuid.as_uuid())
        .execute(&state.db)
        .await
        .map_err(CustomerError::from)?;
    }

    let profile_row = sqlx::query(
        r#"
        INSERT INTO customer_profiles (id, customer_id, store_id, name, email, phone, status, notes)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ON CONFLICT (customer_id, store_id)
        DO UPDATE SET name = EXCLUDED.name,
                      email = EXCLUDED.email,
                      phone = EXCLUDED.phone,
                      status = EXCLUDED.status,
                      notes = EXCLUDED.notes,
                      updated_at = now()
        RETURNING id::text as id, customer_id::text as customer_id, store_id::text as store_id,
                  name, email, phone, status, notes, created_at, updated_at
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(customer_uuid)
    .bind(store_uuid.as_uuid())
    .bind(profile.name)
    .bind(if profile.email.is_empty() { None } else { Some(profile.email) })
    .bind(if profile.phone.is_empty() { None } else { Some(profile.phone) })
    .bind(if profile.status.is_empty() { DEFAULT_PROFILE_STATUS } else { profile.status.as_str() })
    .bind(if profile.notes.is_empty() { None } else { Some(profile.notes) })
    .fetch_one(&state.db)
    .await
    .map_err(CustomerError::from)?;

    let customer_row = sqlx::query(
        r#"
        SELECT id::text as id, tenant_id::text as tenant_id, status, created_at, updated_at
        FROM customers
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(customer_uuid)
    .bind(tenant_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(CustomerError::from)?;

    let customer = pb::Customer {
        id: customer_row.get("id"),
        tenant_id: customer_row.get("tenant_id"),
        status: customer_row.get("status"),
        created_at: chrono_to_timestamp(Some(customer_row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(customer_row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
    };
    let profile = pb::CustomerProfile {
        id: profile_row.get("id"),
        customer_id: profile_row.get("customer_id"),
        store_id: profile_row.get("store_id"),
        name: profile_row.get("name"),
        email: profile_row.get::<Option<String>, _>("email").unwrap_or_default(),
        phone: profile_row.get::<Option<String>, _>("phone").unwrap_or_default(),
        status: profile_row.get("status"),
        notes: profile_row.get::<Option<String>, _>("notes").unwrap_or_default(),
        created_at: chrono_to_timestamp(Some(profile_row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(profile_row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
    };

    let _ = audit::record(
        state,
        audit_input(
            customer.tenant_id.clone(),
            CustomerAuditAction::Update.into(),
            Some("customer_profile"),
            Some(profile.id.clone()),
            None,
            to_json_opt(Some(profile.clone())),
            actor,
        ),
    )
    .await?;

    let _ = outbox::enqueue(
        state,
        outbox::OutboxEventInput {
            tenant_id: customer.tenant_id.clone(),
            store_id: Some(store_id.clone()),
            aggregate_type: "customer".to_string(),
            aggregate_id: customer.id.clone(),
            event_type: "customer.profile_upsert".to_string(),
            payload_json: serde_json::json!({
                "tenant_id": customer.tenant_id,
                "source_store_id": store_id,
                "customer_id": customer.id,
                "profile": {
                    "name": profile.name,
                    "email": profile.email,
                    "phone": profile.phone,
                    "status": profile.status,
                    "notes": profile.notes,
                }
            }),
        },
    )
    .await?;

    Ok((customer, profile))
}

pub async fn upsert_customer_identity(
    state: &AppState,
    tenant_id: String,
    customer_id: String,
    identity: pb::CustomerIdentityUpsert,
    actor: Option<pb::ActorContext>,
) -> CustomerResult<pb::CustomerIdentity> {
    if identity.identity_type.is_empty() || identity.identity_value.is_empty() {
        return Err(CustomerError::InvalidArgument(
            "identity_type and identity_value are required".to_string(),
        ));
    }
    let tenant_uuid = TenantId::parse(&tenant_id).map_err(CustomerError::from)?;
    let customer_uuid = parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?;
    let normalized_value = normalize_identity(&identity.identity_type, &identity.identity_value);

    let existing = sqlx::query(
        r#"
        SELECT id::text as id, customer_id::text as customer_id, verified, source, created_at
        FROM customer_identities
        WHERE tenant_id = $1 AND identity_type = $2 AND identity_value = $3
        "#,
    )
    .bind(tenant_uuid.as_uuid())
    .bind(&identity.identity_type)
    .bind(&normalized_value)
    .fetch_optional(&state.db)
    .await
    .map_err(CustomerError::from)?;

    if let Some(row) = existing {
        let existing_customer_id: String = row.get("customer_id");
        if existing_customer_id != customer_id {
            return Err(CustomerError::AlreadyExists(
                "identity is already linked to another customer".to_string(),
            ));
        }
        sqlx::query(
            r#"
            UPDATE customer_identities
            SET verified = $1, source = $2
            WHERE id = $3
            "#,
        )
        .bind(identity.verified)
        .bind(if identity.source.is_empty() { "admin" } else { identity.source.as_str() })
        .bind(parse_uuid(&row.get::<String, _>("id"), "identity_id").map_err(CustomerError::from)?)
        .execute(&state.db)
        .await
        .map_err(CustomerError::from)?;

        let updated = pb::CustomerIdentity {
            id: row.get("id"),
            customer_id: existing_customer_id,
            tenant_id,
            identity_type: identity.identity_type,
            identity_value: normalized_value,
            verified: identity.verified,
            source: row.get::<String, _>("source"),
            created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        };

        let _ = audit::record(
            state,
            audit_input(
                updated.tenant_id.clone(),
                CustomerAuditAction::IdentityUpsert.into(),
                Some("customer_identity"),
                Some(updated.id.clone()),
                None,
                to_json_opt(Some(updated.clone())),
                actor,
            ),
        )
        .await?;

        let _ = outbox::enqueue(
            state,
            outbox::OutboxEventInput {
                tenant_id: updated.tenant_id.clone(),
                store_id: None,
                aggregate_type: "customer".to_string(),
                aggregate_id: updated.customer_id.clone(),
                event_type: "customer.identity_upsert".to_string(),
                payload_json: serde_json::json!({
                    "tenant_id": updated.tenant_id,
                    "source_store_id": null,
                    "customer_id": updated.customer_id,
                    "identity": {
                        "identity_type": updated.identity_type,
                        "identity_value": updated.identity_value,
                        "verified": updated.verified,
                        "source": updated.source,
                    }
                }),
            },
        )
        .await?;

        return Ok(updated);
    }

    let identity_id = if identity.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&identity.id, "identity_id").map_err(CustomerError::from)?
    };

    sqlx::query(
        r#"
        INSERT INTO customer_identities
            (id, tenant_id, customer_id, identity_type, identity_value, verified, source)
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        "#,
    )
    .bind(identity_id)
    .bind(tenant_uuid.as_uuid())
    .bind(customer_uuid)
    .bind(&identity.identity_type)
    .bind(&normalized_value)
    .bind(identity.verified)
    .bind(if identity.source.is_empty() { "admin" } else { identity.source.as_str() })
    .execute(&state.db)
    .await
    .map_err(CustomerError::from)?;

    let created = pb::CustomerIdentity {
        id: identity_id.to_string(),
        customer_id,
        tenant_id,
        identity_type: identity.identity_type,
        identity_value: normalized_value,
        verified: identity.verified,
        source: if identity.source.is_empty() { "admin".to_string() } else { identity.source },
        created_at: chrono_to_timestamp(Some(Utc::now())),
    };

    let _ = audit::record(
        state,
        audit_input(
            created.tenant_id.clone(),
            CustomerAuditAction::IdentityUpsert.into(),
            Some("customer_identity"),
            Some(created.id.clone()),
            None,
            to_json_opt(Some(created.clone())),
            actor,
        ),
    )
    .await?;

    let _ = outbox::enqueue(
        state,
        outbox::OutboxEventInput {
            tenant_id: created.tenant_id.clone(),
            store_id: None,
            aggregate_type: "customer".to_string(),
            aggregate_id: created.customer_id.clone(),
            event_type: "customer.identity_upsert".to_string(),
            payload_json: serde_json::json!({
                "tenant_id": created.tenant_id,
                "source_store_id": null,
                "customer_id": created.customer_id,
                "identity": {
                    "identity_type": created.identity_type,
                    "identity_value": created.identity_value,
                    "verified": created.verified,
                    "source": created.source,
                }
            }),
        },
    )
    .await?;

    Ok(created)
}

pub async fn upsert_customer_address(
    state: &AppState,
    customer_id: String,
    address: pb::CustomerAddressInput,
    actor: Option<pb::ActorContext>,
) -> CustomerResult<pb::CustomerAddress> {
    if address.r#type.is_empty()
        || address.name.is_empty()
        || address.postal_code.is_empty()
        || address.prefecture.is_empty()
        || address.city.is_empty()
        || address.line1.is_empty()
    {
        return Err(CustomerError::InvalidArgument(
            "address required fields are missing".to_string(),
        ));
    }

    let customer_uuid = parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?;
    let address_id = if address.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&address.id, "address_id").map_err(CustomerError::from)?
    };

    if address.id.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO customer_addresses
                (id, customer_id, type, name, postal_code, prefecture, city, line1, line2, phone)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#,
        )
        .bind(address_id)
        .bind(customer_uuid)
        .bind(&address.r#type)
        .bind(&address.name)
        .bind(&address.postal_code)
        .bind(&address.prefecture)
        .bind(&address.city)
        .bind(&address.line1)
        .bind(if address.line2.is_empty() { None } else { Some(address.line2.clone()) })
        .bind(if address.phone.is_empty() { None } else { Some(address.phone.clone()) })
        .execute(&state.db)
        .await
        .map_err(CustomerError::from)?;
    } else {
        sqlx::query(
            r#"
            UPDATE customer_addresses
            SET type = $1, name = $2, postal_code = $3, prefecture = $4, city = $5,
                line1 = $6, line2 = $7, phone = $8, updated_at = now()
            WHERE id = $9 AND customer_id = $10
            "#,
        )
        .bind(&address.r#type)
        .bind(&address.name)
        .bind(&address.postal_code)
        .bind(&address.prefecture)
        .bind(&address.city)
        .bind(&address.line1)
        .bind(if address.line2.is_empty() { None } else { Some(address.line2.clone()) })
        .bind(if address.phone.is_empty() { None } else { Some(address.phone.clone()) })
        .bind(address_id)
        .bind(customer_uuid)
        .execute(&state.db)
        .await
        .map_err(CustomerError::from)?;
    }

    let updated = pb::CustomerAddress {
        id: address_id.to_string(),
        customer_id,
        r#type: address.r#type,
        name: address.name,
        postal_code: address.postal_code,
        prefecture: address.prefecture,
        city: address.city,
        line1: address.line1,
        line2: address.line2,
        phone: address.phone,
        created_at: None,
        updated_at: None,
    };

    let tenant_id = resolve_tenant_id(state, &updated.customer_id).await?;

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            CustomerAuditAction::AddressUpsert.into(),
            Some("customer_address"),
            Some(updated.id.clone()),
            None,
            to_json_opt(Some(updated.clone())),
            actor,
        ),
    )
    .await?;

    let _ = outbox::enqueue(
        state,
        outbox::OutboxEventInput {
            tenant_id,
            store_id: None,
            aggregate_type: "customer".to_string(),
            aggregate_id: updated.customer_id.clone(),
            event_type: "customer.address_upsert".to_string(),
            payload_json: serde_json::json!({
                "customer_id": updated.customer_id,
                "address_id": updated.id,
            }),
        },
    )
    .await?;

    Ok(updated)
}

fn validate_customer_profile(
    profile: &pb::CustomerProfileInput,
    identities: &Vec<pb::CustomerIdentityInput>,
) -> CustomerResult<()> {
    let has_identifier = !profile.email.is_empty() || !profile.phone.is_empty() || !identities.is_empty();
    if profile.name.is_empty() && !has_identifier {
        return Err(CustomerError::InvalidArgument(
            "customer name or identity is required".to_string(),
        ));
    }
    Ok(())
}

fn normalize_identities(
    profile: &pb::CustomerProfileInput,
    identities: Vec<pb::CustomerIdentityInput>,
) -> Vec<pb::CustomerIdentityInput> {
    let mut result = Vec::new();
    if !profile.email.is_empty() {
        result.push(pb::CustomerIdentityInput {
            identity_type: "email".to_string(),
            identity_value: profile.email.clone(),
            verified: false,
        });
    }
    if !profile.phone.is_empty() {
        result.push(pb::CustomerIdentityInput {
            identity_type: "phone".to_string(),
            identity_value: profile.phone.clone(),
            verified: false,
        });
    }
    result.extend(identities);
    result
        .into_iter()
        .filter(|identity| {
            !identity.identity_type.trim().is_empty() && !identity.identity_value.trim().is_empty()
        })
        .collect()
}

fn normalize_identity(identity_type: &str, value: &str) -> String {
    let trimmed = value.trim();
    match identity_type {
        "email" => trimmed.to_lowercase(),
        "phone" => trimmed.chars().filter(|c| c.is_ascii_digit()).collect(),
        _ => trimmed.to_string(),
    }
}

async fn resolve_tenant_id(
    state: &AppState,
    customer_id: &str,
) -> CustomerResult<String> {
    let row = sqlx::query(
        r#"
        SELECT tenant_id::text as tenant_id
        FROM customers
        WHERE id = $1
        "#,
    )
    .bind(parse_uuid(customer_id, "customer_id").map_err(CustomerError::from)?)
    .fetch_one(&state.db)
    .await
    .map_err(CustomerError::from)?;
    Ok(row.get("tenant_id"))
}
