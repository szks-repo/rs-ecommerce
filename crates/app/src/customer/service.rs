use chrono::Utc;
use sqlx::Row;

use crate::{
    AppState,
    customer::error::{CustomerError, CustomerResult},
    infrastructure::{audit, metafields, outbox},
    pb::pb,
    shared::validation::{Email, Phone},
    shared::{
        audit_action::CustomerAuditAction,
        audit_helpers::{audit_input, to_json_opt},
        ids::{StoreId, TenantId, parse_uuid},
        time::chrono_to_timestamp,
    },
};

const DEFAULT_PROFILE_STATUS: &str = "active";
const DEFAULT_CUSTOMER_STATUS: &str = "active";
const METAFIELD_OWNER_TYPE_CUSTOMER: &str = "customer";

pub async fn list_customers(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    query: String,
    page: Option<pb::PageInfo>,
) -> CustomerResult<(Vec<pb::CustomerSummary>, pb::PageResult)> {
    let store_uuid = StoreId::parse(&store_id).map_err(CustomerError::from)?;
    let tenant_uuid = TenantId::parse(&tenant_id).map_err(CustomerError::from)?;
    let q = query.trim();
    let (limit, offset) = page_params(page);

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
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_uuid.as_uuid())
        .bind(store_uuid.as_uuid())
        .bind(limit)
        .bind(offset)
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
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(tenant_uuid.as_uuid())
        .bind(store_uuid.as_uuid())
        .bind(pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(CustomerError::from)?
    };

    let customers: Vec<pb::CustomerSummary> = rows
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

    let mut next_page_token = String::new();
    if (customers.len() as i64) == limit {
        next_page_token = (offset + limit).to_string();
    }
    Ok((customers, pb::PageResult { next_page_token }))
}

fn page_params(page: Option<pb::PageInfo>) -> (i64, i64) {
    let page = page.unwrap_or(pb::PageInfo {
        page_size: 50,
        page_token: String::new(),
    });
    let limit = (page.page_size.max(1).min(200)) as i64;
    let offset = page.page_token.parse::<i64>().unwrap_or(0).max(0);
    (limit, offset)
}

pub async fn get_customer(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    customer_id: String,
) -> CustomerResult<(
    pb::Customer,
    pb::CustomerProfile,
    Vec<pb::CustomerIdentity>,
    Vec<pb::CustomerAddress>,
)> {
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
               name, email, phone, status, notes, country_code, created_at, updated_at
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
            country_code: row
                .get::<Option<String>, _>("country_code")
                .unwrap_or_else(|| "JP".to_string()),
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
            country_code: "JP".to_string(),
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
               postal_code, prefecture, city, line1, line2, phone, country_code,
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
            country_code: row
                .get::<Option<String>, _>("country_code")
                .unwrap_or_else(|| "JP".to_string()),
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
    if profile.country_code.is_empty() {
        profile.country_code = "JP".to_string();
    }
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

    let mut tx = state.db.begin().await.map_err(CustomerError::from)?;
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
        .execute(tx.as_mut())
        .await
        .map_err(CustomerError::from)?;
    }

    let profile_row = sqlx::query(
        r#"
        INSERT INTO customer_profiles (id, customer_id, store_id, name, email, phone, status, notes, country_code)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        ON CONFLICT (customer_id, store_id)
        DO UPDATE SET name = EXCLUDED.name,
                      email = EXCLUDED.email,
                      phone = EXCLUDED.phone,
                      status = EXCLUDED.status,
                      notes = EXCLUDED.notes,
                      country_code = EXCLUDED.country_code,
                      updated_at = now()
        RETURNING id::text as id, customer_id::text as customer_id, store_id::text as store_id,
                  name, email, phone, status, notes, country_code, created_at, updated_at
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?)
    .bind(store_uuid.as_uuid())
    .bind(profile.name)
    .bind(if profile.email.is_empty() {
        None
    } else {
        Some(profile.email)
    })
    .bind(if profile.phone.is_empty() {
        None
    } else {
        Some(profile.phone)
    })
    .bind(if profile.status.is_empty() {
        DEFAULT_PROFILE_STATUS
    } else {
        profile.status.as_str()
    })
    .bind(if profile.notes.is_empty() {
        None
    } else {
        Some(profile.notes)
    })
    .bind(if profile.country_code.is_empty() {
        "JP"
    } else {
        profile.country_code.as_str()
    })
    .fetch_one(tx.as_mut())
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
        .execute(tx.as_mut())
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
        country_code: profile_row
            .get::<Option<String>, _>("country_code")
            .unwrap_or_else(|| "JP".to_string()),
        created_at: chrono_to_timestamp(Some(profile_row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(profile_row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            CustomerAuditAction::Create.into(),
            Some("customer"),
            Some(customer.id.clone()),
            None,
            to_json_opt(Some(profile.clone())),
            actor,
        ),
    )
    .await?;

    outbox::enqueue_tx(
        &mut tx,
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
                    "name": profile.name.clone(),
                    "email": profile.email.clone(),
                    "phone": profile.phone.clone(),
                    "status": profile.status.clone(),
                    "notes": profile.notes.clone(),
                    "country_code": profile.country_code.clone(),
                }
            }),
        },
    )
    .await?;

    for identity in identity_inputs {
        outbox::enqueue_tx(
            &mut tx,
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

    tx.commit().await.map_err(CustomerError::from)?;
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
    if profile.country_code.is_empty() {
        profile.country_code = "JP".to_string();
    }

    let mut tx = state.db.begin().await.map_err(CustomerError::from)?;
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
        .execute(tx.as_mut())
        .await
        .map_err(CustomerError::from)?;
    }

    let profile_row = sqlx::query(
        r#"
        INSERT INTO customer_profiles (id, customer_id, store_id, name, email, phone, status, notes, country_code)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        ON CONFLICT (customer_id, store_id)
        DO UPDATE SET name = EXCLUDED.name,
                      email = EXCLUDED.email,
                      phone = EXCLUDED.phone,
                      status = EXCLUDED.status,
                      notes = EXCLUDED.notes,
                      country_code = EXCLUDED.country_code,
                      updated_at = now()
        RETURNING id::text as id, customer_id::text as customer_id, store_id::text as store_id,
                  name, email, phone, status, notes, country_code, created_at, updated_at
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(customer_uuid)
    .bind(store_uuid.as_uuid())
    .bind(profile.name)
    .bind(if profile.email.is_empty() {
        None
    } else {
        Some(profile.email)
    })
    .bind(if profile.phone.is_empty() {
        None
    } else {
        Some(profile.phone)
    })
    .bind(if profile.status.is_empty() {
        DEFAULT_PROFILE_STATUS
    } else {
        profile.status.as_str()
    })
    .bind(if profile.notes.is_empty() {
        None
    } else {
        Some(profile.notes)
    })
    .bind(if profile.country_code.is_empty() {
        "JP"
    } else {
        profile.country_code.as_str()
    })
    .fetch_one(tx.as_mut())
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
    .fetch_one(tx.as_mut())
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
        country_code: profile_row
            .get::<Option<String>, _>("country_code")
            .unwrap_or_else(|| "JP".to_string()),
        created_at: chrono_to_timestamp(Some(profile_row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(profile_row.get::<chrono::DateTime<Utc>, _>("updated_at"))),
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            CustomerAuditAction::Update.into(),
            Some("customer_profile"),
            Some(profile.id.clone()),
            None,
            to_json_opt(Some(profile.clone())),
            actor,
        ),
    )
    .await?;

    outbox::enqueue_tx(
        &mut tx,
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
                    "name": profile.name.clone(),
                    "email": profile.email.clone(),
                    "phone": profile.phone.clone(),
                    "status": profile.status.clone(),
                    "notes": profile.notes.clone(),
                    "country_code": profile.country_code.clone(),
                }
            }),
        },
    )
    .await?;

    tx.commit().await.map_err(CustomerError::from)?;
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

    let mut tx = state.db.begin().await.map_err(CustomerError::from)?;
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
    .fetch_optional(tx.as_mut())
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
        .bind(if identity.source.is_empty() {
            "admin"
        } else {
            identity.source.as_str()
        })
        .bind(parse_uuid(&row.get::<String, _>("id"), "identity_id").map_err(CustomerError::from)?)
        .execute(tx.as_mut())
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

        audit::record_tx(
            &mut tx,
            audit_input(
                None,
                CustomerAuditAction::IdentityUpsert.into(),
                Some("customer_identity"),
                Some(updated.id.clone()),
                None,
                to_json_opt(Some(updated.clone())),
                actor,
            ),
        )
        .await?;

        outbox::enqueue_tx(
            &mut tx,
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
                        "identity_type": updated.identity_type.clone(),
                        "identity_value": updated.identity_value.clone(),
                        "verified": updated.verified,
                        "source": updated.source.clone(),
                    }
                }),
            },
        )
        .await?;

        tx.commit().await.map_err(CustomerError::from)?;
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
    .bind(if identity.source.is_empty() {
        "admin"
    } else {
        identity.source.as_str()
    })
    .execute(tx.as_mut())
    .await
    .map_err(CustomerError::from)?;

    let created = pb::CustomerIdentity {
        id: identity_id.to_string(),
        customer_id,
        tenant_id,
        identity_type: identity.identity_type,
        identity_value: normalized_value,
        verified: identity.verified,
        source: if identity.source.is_empty() {
            "admin".to_string()
        } else {
            identity.source
        },
        created_at: chrono_to_timestamp(Some(Utc::now())),
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            None,
            CustomerAuditAction::IdentityUpsert.into(),
            Some("customer_identity"),
            Some(created.id.clone()),
            None,
            to_json_opt(Some(created.clone())),
            actor,
        ),
    )
    .await?;

    outbox::enqueue_tx(
        &mut tx,
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
                    "identity_type": created.identity_type.clone(),
                    "identity_value": created.identity_value.clone(),
                    "verified": created.verified,
                    "source": created.source.clone(),
                }
            }),
        },
    )
    .await?;

    tx.commit().await.map_err(CustomerError::from)?;
    Ok(created)
}

pub async fn upsert_customer_address(
    state: &AppState,
    customer_id: String,
    mut address: pb::CustomerAddressInput,
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
    if address.country_code.is_empty() {
        address.country_code = "JP".to_string();
    }

    let customer_uuid = parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?;
    let address_id = if address.id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        parse_uuid(&address.id, "address_id").map_err(CustomerError::from)?
    };

    let mut tx = state.db.begin().await.map_err(CustomerError::from)?;
    if address.id.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO customer_addresses
                (id, customer_id, type, name, postal_code, prefecture, city, line1, line2, phone, country_code)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
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
        .bind(if address.line2.is_empty() {
            None
        } else {
            Some(address.line2.clone())
        })
        .bind(if address.phone.is_empty() {
            None
        } else {
            Some(address.phone.clone())
        })
        .bind(if address.country_code.is_empty() {
            "JP"
        } else {
            address.country_code.as_str()
        })
        .execute(tx.as_mut())
        .await
        .map_err(CustomerError::from)?;
    } else {
        sqlx::query(
            r#"
            UPDATE customer_addresses
            SET type = $1, name = $2, postal_code = $3, prefecture = $4, city = $5,
                line1 = $6, line2 = $7, phone = $8, country_code = $9, updated_at = now()
            WHERE id = $10 AND customer_id = $11
            "#,
        )
        .bind(&address.r#type)
        .bind(&address.name)
        .bind(&address.postal_code)
        .bind(&address.prefecture)
        .bind(&address.city)
        .bind(&address.line1)
        .bind(if address.line2.is_empty() {
            None
        } else {
            Some(address.line2.clone())
        })
        .bind(if address.phone.is_empty() {
            None
        } else {
            Some(address.phone.clone())
        })
        .bind(if address.country_code.is_empty() {
            "JP"
        } else {
            address.country_code.as_str()
        })
        .bind(address_id)
        .bind(customer_uuid)
        .execute(tx.as_mut())
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
        country_code: if address.country_code.is_empty() {
            "JP".to_string()
        } else {
            address.country_code
        },
        created_at: None,
        updated_at: None,
    };

    let tenant_row = sqlx::query(
        r#"
        SELECT tenant_id::text as tenant_id
        FROM customers
        WHERE id = $1
        "#,
    )
    .bind(customer_uuid)
    .fetch_one(tx.as_mut())
    .await
    .map_err(CustomerError::from)?;
    let tenant_id: String = tenant_row.get("tenant_id");

    audit::record_tx(
        &mut tx,
        audit_input(
            None,
            CustomerAuditAction::AddressUpsert.into(),
            Some("customer_address"),
            Some(updated.id.clone()),
            None,
            to_json_opt(Some(updated.clone())),
            actor,
        ),
    )
    .await?;

    outbox::enqueue_tx(
        &mut tx,
        outbox::OutboxEventInput {
            tenant_id,
            store_id: None,
            aggregate_type: "customer".to_string(),
            aggregate_id: updated.customer_id.clone(),
            event_type: "customer.address_upsert".to_string(),
            payload_json: serde_json::json!({
                "customer_id": updated.customer_id,
                "address_id": updated.id.clone(),
            }),
        },
    )
    .await?;

    tx.commit().await.map_err(CustomerError::from)?;
    Ok(updated)
}

fn metafield_definition_from_record(record: &metafields::MetafieldDefinitionRecord) -> pb::MetafieldDefinition {
    pb::MetafieldDefinition {
        id: record.id.clone(),
        owner_type: record.owner_type.clone(),
        namespace: record.namespace.clone(),
        key: record.key.clone(),
        name: record.name.clone(),
        description: record.description.clone(),
        value_type: record.value_type.clone(),
        is_list: record.is_list,
        validations_json: record.validations_json.clone(),
        visibility_json: record.visibility_json.clone(),
        created_at: chrono_to_timestamp(Some(record.created_at)),
        updated_at: chrono_to_timestamp(Some(record.updated_at)),
    }
}

pub async fn list_customer_metafield_definitions(state: &AppState) -> CustomerResult<Vec<pb::MetafieldDefinition>> {
    let definitions = metafields::list_definitions(&state.db, METAFIELD_OWNER_TYPE_CUSTOMER).await?;
    Ok(definitions
        .into_iter()
        .map(|record| metafield_definition_from_record(&record))
        .collect())
}

pub async fn create_customer_metafield_definition(
    state: &AppState,
    input: pb::CustomerMetafieldDefinitionInput,
) -> CustomerResult<pb::MetafieldDefinition> {
    if input.namespace.is_empty() || input.key.is_empty() || input.name.is_empty() || input.value_type.is_empty() {
        return Err(CustomerError::InvalidArgument(
            "definition required fields are missing".to_string(),
        ));
    }

    let validations_json = metafields::normalize_optional_json(input.validations_json).map_err(CustomerError::from)?;
    let visibility_json = metafields::normalize_optional_json(input.visibility_json).map_err(CustomerError::from)?;

    let record = metafields::create_definition(
        &state.db,
        METAFIELD_OWNER_TYPE_CUSTOMER,
        metafields::MetafieldDefinitionInput {
            namespace: input.namespace,
            key: input.key,
            name: input.name,
            description: if input.description.is_empty() {
                None
            } else {
                Some(input.description)
            },
            value_type: input.value_type,
            is_list: input.is_list,
            validations_json,
            visibility_json,
        },
    )
    .await
    .map_err(CustomerError::from)?;

    Ok(metafield_definition_from_record(&record))
}

pub async fn update_customer_metafield_definition(
    state: &AppState,
    definition_id: String,
    input: pb::CustomerMetafieldDefinitionInput,
) -> CustomerResult<pb::MetafieldDefinition> {
    if definition_id.trim().is_empty() {
        return Err(CustomerError::InvalidArgument("definition_id is required".to_string()));
    }
    if input.namespace.is_empty() || input.key.is_empty() || input.name.is_empty() || input.value_type.is_empty() {
        return Err(CustomerError::InvalidArgument(
            "definition required fields are missing".to_string(),
        ));
    }

    let definition_uuid = parse_uuid(&definition_id, "definition_id").map_err(CustomerError::from)?;
    let validations_json = metafields::normalize_optional_json(input.validations_json).map_err(CustomerError::from)?;
    let visibility_json = metafields::normalize_optional_json(input.visibility_json).map_err(CustomerError::from)?;

    let record = metafields::update_definition(
        &state.db,
        METAFIELD_OWNER_TYPE_CUSTOMER,
        &definition_uuid,
        metafields::MetafieldDefinitionInput {
            namespace: input.namespace,
            key: input.key,
            name: input.name,
            description: if input.description.is_empty() {
                None
            } else {
                Some(input.description)
            },
            value_type: input.value_type,
            is_list: input.is_list,
            validations_json,
            visibility_json,
        },
    )
    .await
    .map_err(CustomerError::from)?;

    let Some(record) = record else {
        return Err(CustomerError::NotFound("metafield definition not found".to_string()));
    };

    Ok(metafield_definition_from_record(&record))
}

pub async fn list_customer_metafield_values(
    state: &AppState,
    tenant_id: String,
    customer_id: String,
) -> CustomerResult<Vec<pb::MetafieldValue>> {
    let tenant_uuid = TenantId::parse(&tenant_id).map_err(CustomerError::from)?;
    let customer_uuid = parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?;

    let exists = sqlx::query(
        r#"
        SELECT 1
        FROM customers
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(customer_uuid)
    .bind(tenant_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(CustomerError::from)?;

    if exists.is_none() {
        return Err(CustomerError::NotFound("customer not found".to_string()));
    }

    let records = metafields::list_values(&state.db, METAFIELD_OWNER_TYPE_CUSTOMER, &customer_uuid)
        .await
        .map_err(CustomerError::from)?;

    let values = records
        .into_iter()
        .map(|record| pb::MetafieldValue {
            id: record.id,
            definition_id: record.definition_id,
            owner_id: record.owner_id,
            value_json: record.value_json,
            created_at: chrono_to_timestamp(Some(record.created_at)),
            updated_at: chrono_to_timestamp(Some(record.updated_at)),
            definition: Some(metafield_definition_from_record(&record.definition)),
        })
        .collect();

    Ok(values)
}

pub async fn upsert_customer_metafield_value(
    state: &AppState,
    tenant_id: String,
    customer_id: String,
    definition_id: String,
    value_json: String,
    _actor: Option<pb::ActorContext>,
) -> CustomerResult<()> {
    let tenant_uuid = TenantId::parse(&tenant_id).map_err(CustomerError::from)?;
    let customer_uuid = parse_uuid(&customer_id, "customer_id").map_err(CustomerError::from)?;
    let definition_uuid = parse_uuid(&definition_id, "definition_id").map_err(CustomerError::from)?;

    if value_json.trim().is_empty() {
        return Err(CustomerError::InvalidArgument("value_json is required".to_string()));
    }

    let value = serde_json::from_str::<serde_json::Value>(&value_json)
        .map_err(|_| CustomerError::InvalidArgument("value_json must be valid JSON".to_string()))?;

    let exists = sqlx::query(
        r#"
        SELECT 1
        FROM customers
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(customer_uuid)
    .bind(tenant_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(CustomerError::from)?;

    if exists.is_none() {
        return Err(CustomerError::NotFound("customer not found".to_string()));
    }

    let definition = metafields::fetch_definition(&state.db, METAFIELD_OWNER_TYPE_CUSTOMER, &definition_uuid)
        .await
        .map_err(CustomerError::from)?;

    let Some(definition) = definition else {
        return Err(CustomerError::NotFound("metafield definition not found".to_string()));
    };
    let value_type = definition.value_type.clone();
    let is_list = definition.is_list;
    if value_type == "bool" || value_type == "boolean" {
        let valid = if is_list {
            value.is_boolean()
                || value
                    .as_array()
                    .map(|items| items.iter().all(|item| item.is_boolean()))
                    .unwrap_or(false)
        } else {
            value.is_boolean()
        };
        if !valid {
            return Err(CustomerError::InvalidArgument(
                "value_json must be a boolean".to_string(),
            ));
        }
    }
    let _definition = metafield_definition_from_record(&definition);

    metafields::upsert_value(&state.db, &definition_uuid, &customer_uuid, &value_json)
        .await
        .map_err(CustomerError::from)?;

    Ok(())
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
        .filter(|identity| !identity.identity_type.trim().is_empty() && !identity.identity_value.trim().is_empty())
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
