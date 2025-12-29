use axum::{Json, http::StatusCode};
use sqlx::Row;
use serde_json::Value;

use crate::{
    AppState,
    pb::pb,
    infrastructure::{db, audit},
    rpc::json::ConnectError,
    shared::{ids::parse_uuid, money::{money_from_parts, money_to_parts, money_to_parts_opt}},
};
use crate::rpc::request_context;

pub async fn list_products(
    state: &AppState,
    tenant_id: String,
) -> Result<Vec<pb::Product>, (StatusCode, Json<ConnectError>)> {
    let store_id = store_id_for_tenant(state, &tenant_id).await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id, vendor_id::text as vendor_id, title, description, status
        FROM products
        WHERE tenant_id = $1 AND store_id = $2
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(&tenant_id)
    .bind(parse_uuid(&store_id, "store_id")?)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::Product {
            id: row.get::<String, _>("id"),
            vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            variants: Vec::new(),
            updated_at: None,
        })
        .collect())
}

pub async fn get_product(
    state: &AppState,
    tenant_id: String,
    product_id: String,
) -> Result<Option<pb::Product>, (StatusCode, Json<ConnectError>)> {
    let store_id = store_id_for_tenant(state, &tenant_id).await?;
    let row = sqlx::query(
        r#"
        SELECT id::text as id, vendor_id::text as vendor_id, title, description, status
        FROM products
        WHERE tenant_id = $1 AND store_id = $2 AND id = $3
        "#,
    )
    .bind(&tenant_id)
    .bind(parse_uuid(&store_id, "store_id")?)
    .bind(parse_uuid(&product_id, "product_id")?)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    Ok(row.map(|row| pb::Product {
        id: row.get::<String, _>("id"),
        vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
        title: row.get("title"),
        description: row.get("description"),
        status: row.get("status"),
        variants: Vec::new(),
        updated_at: None,
    }))
}

pub async fn list_products_admin(
    state: &AppState,
    tenant: Option<pb::TenantContext>,
    store: Option<pb::StoreContext>,
) -> Result<Vec<pb::ProductAdmin>, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, store, tenant).await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id, store_id::text as store_id, vendor_id::text as vendor_id, title, description, status
        FROM products
        WHERE tenant_id = $1 AND store_id = $2
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(&tenant_id)
    .bind(parse_uuid(&store_id, "store_id")?)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::ProductAdmin {
            id: row.get::<String, _>("id"),
            vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            updated_at: None,
            store_id: row.get::<String, _>("store_id"),
        })
        .collect())
}

pub async fn list_variants_admin(
    state: &AppState,
    tenant: Option<pb::TenantContext>,
    store: Option<pb::StoreContext>,
    product_id: String,
) -> Result<Vec<pb::VariantAdmin>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    let rows = sqlx::query(
        r#"
        SELECT v.id::text as id,
               v.product_id::text as product_id,
               v.sku,
               v.fulfillment_type,
               v.price_amount,
               v.price_currency,
               v.compare_at_amount,
               v.compare_at_currency,
               v.status
        FROM variants v
        JOIN products p ON p.id = v.product_id
        WHERE p.store_id = $1 AND v.product_id = $2
        ORDER BY v.created_at DESC
        "#,
    )
    .bind(parse_uuid(&store_id, "store_id")?)
    .bind(parse_uuid(&product_id, "product_id")?)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::VariantAdmin {
            id: row.get("id"),
            product_id: row.get("product_id"),
            sku: row.get("sku"),
            fulfillment_type: row.get("fulfillment_type"),
            price: Some(money_from_parts(
                row.get::<i64, _>("price_amount"),
                row.get::<String, _>("price_currency"),
            )),
            compare_at: match row.get::<Option<i64>, _>("compare_at_amount") {
                Some(amount) => Some(money_from_parts(
                    amount,
                    row.get::<Option<String>, _>("compare_at_currency").unwrap_or_default(),
                )),
                None => None,
            },
            status: row.get("status"),
        })
        .collect())
}

pub async fn create_product(
    state: &AppState,
    req: pb::CreateProductRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::ProductAdmin, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store.clone(), req.tenant.clone()).await?;
    let product_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO products (id, tenant_id, store_id, vendor_id, title, description, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(product_id)
    .bind(&tenant_id)
    .bind(parse_uuid(&store_id, "store_id")?)
    .bind(crate::shared::ids::nullable_uuid(req.vendor_id.clone()))
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.status)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let product = pb::ProductAdmin {
        id: product_id.to_string(),
        vendor_id: req.vendor_id,
        title: req.title,
        description: req.description,
        status: req.status,
        updated_at: None,
        store_id,
    };

    let _ = state
        .search
        .upsert_products(&[crate::infrastructure::search::SearchProduct {
            id: product.id.clone(),
            tenant_id: tenant_id.clone(),
            vendor_id: product.vendor_id.clone(),
            title: product.title.clone(),
            description: product.description.clone(),
            status: product.status.clone(),
        }])
        .await;

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            "product.create",
            Some("product"),
            Some(product.id.clone()),
            None,
            to_json_opt(Some(product.clone())),
            _actor,
        ),
    )
    .await?;

    Ok(product)
}

pub async fn update_product(
    state: &AppState,
    req: pb::UpdateProductRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::ProductAdmin, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store.clone(), req.tenant.clone()).await?;
    let before = fetch_product_admin(state, &tenant_id, &store_id, &req.product_id).await.ok();
    sqlx::query(
        r#"
        UPDATE products
        SET title = $1, description = $2, status = $3, updated_at = now()
        WHERE id = $4 AND tenant_id = $5 AND store_id = $6
        "#,
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.status)
    .bind(parse_uuid(&req.product_id, "product_id")?)
    .bind(&tenant_id)
    .bind(parse_uuid(&store_id, "store_id")?)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let product = pb::ProductAdmin {
        id: req.product_id,
        vendor_id: String::new(),
        title: req.title,
        description: req.description,
        status: req.status,
        updated_at: None,
        store_id: store_id.clone(),
    };

    let mut after = product.clone();
    if let Ok(row) = sqlx::query(
        r#"
        SELECT id::text as id, store_id::text as store_id, vendor_id::text as vendor_id, title, description, status
        FROM products
        WHERE tenant_id = $1 AND store_id = $2 AND id = $3
        "#,
    )
    .bind(&tenant_id)
    .bind(parse_uuid(&store_id, "store_id")?)
    .bind(parse_uuid(&product.id, "product_id")?)
    .fetch_one(&state.db)
    .await
        {
        let _ = state
            .search
            .upsert_products(&[crate::infrastructure::search::SearchProduct {
                id: row.get::<String, _>("id"),
                tenant_id: tenant_id.clone(),
                vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
                title: row.get("title"),
                description: row.get("description"),
                status: row.get("status"),
            }])
            .await;
        after = pb::ProductAdmin {
            id: row.get::<String, _>("id"),
            vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            updated_at: None,
            store_id: row.get::<String, _>("store_id"),
        };
    }

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            "product.update",
            Some("product"),
            Some(product.id.clone()),
            to_json_opt(before),
            to_json_opt(Some(after)),
            _actor,
        ),
    )
    .await?;

    Ok(product)
}

pub async fn create_variant(
    state: &AppState,
    req: pb::CreateVariantRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::VariantAdmin, (StatusCode, Json<ConnectError>)> {
    let tenant_id = tenant_id_for_product(state, &req.product_id).await?;
    if let Some(tenant) = req.tenant.as_ref().and_then(|t| if t.tenant_id.is_empty() { None } else { Some(t.tenant_id.as_str()) }) {
        if tenant != tenant_id {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: "invalid_argument",
                    message: "tenant does not match product".to_string(),
                }),
            ));
        }
    }
    let variant_id = uuid::Uuid::new_v4();
    let (price_amount, price_currency) = money_to_parts(req.price.clone())?;
    let (compare_amount, compare_currency) = money_to_parts_opt(req.compare_at.clone())?;
    let fulfillment_type = normalize_fulfillment_type(req.fulfillment_type.clone())?;
    sqlx::query(
        r#"
        INSERT INTO variants (
            id, product_id, sku, fulfillment_type, price_amount, price_currency,
            compare_at_amount, compare_at_currency, status
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        "#,
    )
    .bind(variant_id)
    .bind(parse_uuid(&req.product_id, "product_id")?)
    .bind(&req.sku)
    .bind(&fulfillment_type)
    .bind(price_amount)
    .bind(&price_currency)
    .bind(compare_amount)
    .bind(compare_currency)
    .bind(&req.status)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    let _ = reindex_product_by_id(state, &req.product_id).await;

    let variant = pb::VariantAdmin {
        id: variant_id.to_string(),
        product_id: req.product_id,
        sku: req.sku,
        fulfillment_type,
        price: req.price,
        compare_at: req.compare_at,
        status: req.status,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id,
            "variant.create",
            Some("variant"),
            Some(variant.id.clone()),
            None,
            to_json_opt(Some(variant.clone())),
            _actor,
        ),
    )
    .await?;

    Ok(variant)
}

pub async fn update_variant(
    state: &AppState,
    req: pb::UpdateVariantRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::VariantAdmin, (StatusCode, Json<ConnectError>)> {
    let tenant_id = tenant_id_for_variant(state, &req.variant_id).await?;
    if let Some(tenant) = req.tenant.as_ref().and_then(|t| if t.tenant_id.is_empty() { None } else { Some(t.tenant_id.as_str()) }) {
        if tenant != tenant_id {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: "invalid_argument",
                    message: "tenant does not match variant".to_string(),
                }),
            ));
        }
    }
    let (price_amount, price_currency) = money_to_parts(req.price.clone())?;
    let (compare_amount, compare_currency) = money_to_parts_opt(req.compare_at.clone())?;
    let fulfillment_type = if req.fulfillment_type.is_empty() {
        None
    } else {
        Some(normalize_fulfillment_type(req.fulfillment_type.clone())?)
    };
    sqlx::query(
        r#"
        UPDATE variants
        SET price_amount = $1, price_currency = $2,
            compare_at_amount = $3, compare_at_currency = $4,
            status = $5,
            fulfillment_type = COALESCE($6, fulfillment_type),
            updated_at = now()
        WHERE id = $7
        "#,
    )
    .bind(price_amount)
    .bind(&price_currency)
    .bind(compare_amount)
    .bind(compare_currency)
    .bind(&req.status)
    .bind(fulfillment_type.as_deref())
    .bind(parse_uuid(&req.variant_id, "variant_id")?)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    if let Ok(row) = sqlx::query(
        "SELECT product_id::text as product_id FROM variants WHERE id = $1",
    )
    .bind(parse_uuid(&req.variant_id, "variant_id")?)
    .fetch_one(&state.db)
    .await
    {
        let product_id: String = row.get("product_id");
        let _ = reindex_product_by_id(state, &product_id).await;
    }

    let variant = pb::VariantAdmin {
        id: req.variant_id,
        product_id: String::new(),
        sku: String::new(),
        fulfillment_type: fulfillment_type.unwrap_or_default(),
        price: req.price,
        compare_at: req.compare_at,
        status: req.status,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id,
            "variant.update",
            Some("variant"),
            Some(variant.id.clone()),
            None,
            to_json_opt(Some(variant.clone())),
            _actor,
        ),
    )
    .await?;

    Ok(variant)
}

pub async fn set_inventory(
    state: &AppState,
    req: pb::SetInventoryRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::InventoryAdmin, (StatusCode, Json<ConnectError>)> {
    if req.location_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "location_id is required".to_string(),
            }),
        ));
    }
    let (store_id, tenant_id) = resolve_store_context(state, req.store.clone(), req.tenant.clone()).await?;
    ensure_variant_belongs_to_store(state, &req.variant_id, &store_id).await?;
    ensure_location_belongs_to_store(state, &req.location_id, &store_id).await?;
    sqlx::query(
        r#"
        INSERT INTO inventory_stocks (tenant_id, store_id, location_id, variant_id, stock, reserved)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (variant_id, location_id)
        DO UPDATE SET stock = EXCLUDED.stock, reserved = EXCLUDED.reserved, updated_at = now()
        "#,
    )
    .bind(&tenant_id)
    .bind(parse_uuid(&store_id, "store_id")?)
    .bind(parse_uuid(&req.location_id, "location_id")?)
    .bind(parse_uuid(&req.variant_id, "variant_id")?)
    .bind(req.stock)
    .bind(req.reserved)
    .execute(&state.db)
    .await
    .map_err(db::error)?;

    if let Ok(row) = sqlx::query(
        r#"
        SELECT p.id::text as id
        FROM products p
        JOIN variants v ON v.product_id = p.id
        WHERE v.id = $1
        "#,
    )
    .bind(parse_uuid(&req.variant_id, "variant_id")?)
    .fetch_one(&state.db)
    .await
    {
        let product_id: String = row.get("id");
        let _ = reindex_product_by_id(state, &product_id).await;
    }

    let inventory = pb::InventoryAdmin {
        variant_id: req.variant_id,
        location_id: req.location_id,
        stock: req.stock,
        reserved: req.reserved,
        updated_at: None,
    };

    let _ = audit::record(
        state,
        audit_input(
            tenant_id.clone(),
            "inventory.set",
            Some("inventory"),
            Some(inventory.variant_id.clone()),
            None,
            to_json_opt(Some(inventory.clone())),
            _actor,
        ),
    )
    .await?;

    Ok(inventory)
}

fn audit_input(
    tenant_id: String,
    action: &str,
    target_type: Option<&str>,
    target_id: Option<String>,
    before_json: Option<Value>,
    after_json: Option<Value>,
    actor: Option<pb::ActorContext>,
) -> audit::AuditInput {
    let (actor_id, actor_type) = actor_fields(actor);
    audit::AuditInput {
        tenant_id,
        actor_id,
        actor_type,
        action: action.to_string(),
        target_type: target_type.map(|v| v.to_string()),
        target_id,
        request_id: None,
        ip_address: None,
        user_agent: None,
        before_json,
        after_json,
        metadata_json: None,
    }
}

fn actor_fields(actor: Option<pb::ActorContext>) -> (Option<String>, String) {
    let actor_id = actor
        .as_ref()
        .and_then(|a| if a.actor_id.is_empty() { None } else { Some(a.actor_id.clone()) });
    let actor_type = actor
        .and_then(|a| if a.actor_type.is_empty() { None } else { Some(a.actor_type) })
        .unwrap_or_else(|| "system".to_string());
    (actor_id, actor_type)
}

fn to_json_opt<T: serde::Serialize>(value: Option<T>) -> Option<Value> {
    value.and_then(|v| serde_json::to_value(v).ok())
}

async fn fetch_product_admin(
    state: &AppState,
    tenant_id: &str,
    store_id: &str,
    product_id: &str,
) -> Result<pb::ProductAdmin, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT id::text as id, store_id::text as store_id, vendor_id::text as vendor_id, title, description, status
        FROM products
        WHERE tenant_id = $1 AND store_id = $2 AND id = $3
        "#,
    )
    .bind(tenant_id)
    .bind(parse_uuid(store_id, "store_id")?)
    .bind(parse_uuid(product_id, "product_id")?)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::ProductAdmin {
        id: row.get("id"),
        vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
        title: row.get("title"),
        description: row.get("description"),
        status: row.get("status"),
        updated_at: None,
        store_id: row.get("store_id"),
    })
}

async fn tenant_id_for_product(
    state: &AppState,
    product_id: &str,
) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM products WHERE id = $1")
        .bind(parse_uuid(product_id, "product_id")?)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
    Ok(row.get("tenant_id"))
}

async fn tenant_id_for_variant(
    state: &AppState,
    variant_id: &str,
) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT p.tenant_id::text as tenant_id
        FROM products p
        JOIN variants v ON v.product_id = p.id
        WHERE v.id = $1
        "#,
    )
    .bind(parse_uuid(variant_id, "variant_id")?)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    Ok(row.get("tenant_id"))
}

async fn store_id_for_tenant(
    state: &AppState,
    tenant_id: &str,
) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query("SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1")
        .bind(parse_uuid(tenant_id, "tenant_id")?)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
    Ok(row.get("id"))
}

async fn resolve_store_context(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> Result<(String, String), (StatusCode, Json<ConnectError>)> {
    if let Some(ctx) = request_context::current() {
        if let Some(auth_store) = ctx.store_id.as_deref() {
            if let Some(store_id) =
                store.as_ref().and_then(|s| if s.store_id.is_empty() { None } else { Some(s.store_id.as_str()) })
            {
                if store_id != auth_store {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(ConnectError {
                            code: "permission_denied",
                            message: "store_id does not match token".to_string(),
                        }),
                    ));
                }
            }
        }
        if let Some(auth_tenant) = ctx.tenant_id.as_deref() {
            if let Some(tenant_id) =
                tenant.as_ref().and_then(|t| if t.tenant_id.is_empty() { None } else { Some(t.tenant_id.as_str()) })
            {
                if tenant_id != auth_tenant {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(ConnectError {
                            code: "permission_denied",
                            message: "tenant_id does not match token".to_string(),
                        }),
                    ));
                }
            }
        }
    }

    if let Some(store_id) = store.and_then(|s| if s.store_id.is_empty() { None } else { Some(s.store_id) }) {
        let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
            .bind(parse_uuid(&store_id, "store_id")?)
            .fetch_one(&state.db)
            .await
            .map_err(db::error)?;
        let tenant_id: String = row.get("tenant_id");
        return Ok((store_id, tenant_id));
    }
    if let Some(tenant_id) = tenant.and_then(|t| if t.tenant_id.is_empty() { None } else { Some(t.tenant_id) }) {
        let store_id = store_id_for_tenant(state, &tenant_id).await?;
        return Ok((store_id, tenant_id));
    }
    if let Some(ctx) = request_context::current() {
        if let Some(store_id) = ctx.store_id {
            let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
                .bind(parse_uuid(&store_id, "store_id")?)
                .fetch_optional(&state.db)
                .await
                .map_err(db::error)?;
            if let Some(row) = row {
                let tenant_id: String = row.get("tenant_id");
                return Ok((store_id, tenant_id));
            }
        }
        if let Some(tenant_id) = ctx.tenant_id {
            let store_id = store_id_for_tenant(state, &tenant_id).await?;
            return Ok((store_id, tenant_id));
        }
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ConnectError {
            code: "invalid_argument",
            message: "store.store_id or tenant.tenant_id is required".to_string(),
        }),
    ))
}

async fn ensure_variant_belongs_to_store(
    state: &AppState,
    variant_id: &str,
    store_id: &str,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT p.store_id::text as store_id
        FROM variants v
        JOIN products p ON p.id = v.product_id
        WHERE v.id = $1
        "#,
    )
    .bind(parse_uuid(variant_id, "variant_id")?)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    let owner_store_id: String = row.get("store_id");
    if owner_store_id != store_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "variant does not belong to store".to_string(),
            }),
        ));
    }
    Ok(())
}

async fn ensure_location_belongs_to_store(
    state: &AppState,
    location_id: &str,
    store_id: &str,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query("SELECT store_id::text as store_id FROM store_locations WHERE id = $1")
        .bind(parse_uuid(location_id, "location_id")?)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
    let owner_store_id: String = row.get("store_id");
    if owner_store_id != store_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: "invalid_argument",
                message: "location does not belong to store".to_string(),
            }),
        ));
    }
    Ok(())
}

fn normalize_fulfillment_type(value: String) -> Result<String, (StatusCode, Json<ConnectError>)> {
    if value.is_empty() || value == "physical" {
        return Ok("physical".to_string());
    }
    if value == "digital" {
        return Ok("digital".to_string());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ConnectError {
            code: "invalid_argument",
            message: "fulfillment_type must be 'physical' or 'digital'".to_string(),
        }),
    ))
}

async fn reindex_product_by_id(
    state: &AppState,
    product_id: &str,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT id::text as id, tenant_id::text as tenant_id, vendor_id::text as vendor_id,
               title, description, status
        FROM products
        WHERE id = $1
        "#,
    )
    .bind(parse_uuid(product_id, "product_id")?)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    state
        .search
        .upsert_products(&[crate::infrastructure::search::SearchProduct {
            id: row.get::<String, _>("id"),
            tenant_id: row.get::<String, _>("tenant_id"),
            vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
        }])
        .await?;
    Ok(())
}
