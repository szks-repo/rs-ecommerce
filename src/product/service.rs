use axum::{Json, http::StatusCode};
use sqlx::Row;

use crate::rpc::request_context;
use crate::{
    AppState,
    infrastructure::{audit, db, metafields},
    pb::pb,
    product::domain::SkuCode,
    rpc::json::ConnectError,
    shared::{
        audit_action::{InventoryAuditAction, ProductAuditAction, VariantAuditAction},
        audit_helpers::{audit_input, to_json_opt},
        ids::{ProductId, StoreId, TenantId, nullable_uuid, parse_uuid},
        money::{money_from_parts, money_to_parts, money_to_parts_opt},
        status::{FulfillmentType, ProductStatus, VariantStatus},
        time::{chrono_to_timestamp, timestamp_to_chrono},
    },
};

const METAFIELD_OWNER_TYPE_PRODUCT: &str = "product";

pub async fn list_products(
    state: &AppState,
    tenant_id: String,
) -> Result<Vec<pb::Product>, (StatusCode, Json<ConnectError>)> {
    let tenant_id = TenantId::parse(&tenant_id)?;
    let store_id = store_id_for_tenant(state, &tenant_id.to_string()).await?;
    let store_id = StoreId::parse(&store_id)?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id, vendor_id::text as vendor_id, title, description, status, tax_rule_id::text as tax_rule_id
        FROM products
        WHERE tenant_id = $1 AND store_id = $2
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(store_id.as_uuid())
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
            tax_rule_id: row.get::<Option<String>, _>("tax_rule_id").unwrap_or_default(),
        })
        .collect())
}

pub async fn get_product(
    state: &AppState,
    tenant_id: String,
    product_id: String,
) -> Result<Option<pb::Product>, (StatusCode, Json<ConnectError>)> {
    let tenant_id = TenantId::parse(&tenant_id)?;
    let store_id = store_id_for_tenant(state, &tenant_id.to_string()).await?;
    let store_id = StoreId::parse(&store_id)?;
    let product_id = ProductId::parse(&product_id)?;
    let row = sqlx::query(
        r#"
        SELECT id::text as id, vendor_id::text as vendor_id, title, description, status, tax_rule_id::text as tax_rule_id
        FROM products
        WHERE tenant_id = $1 AND store_id = $2 AND id = $3
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(store_id.as_uuid())
    .bind(product_id.as_uuid())
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
        tax_rule_id: row.get::<Option<String>, _>("tax_rule_id").unwrap_or_default(),
    }))
}

pub async fn list_products_admin(
    state: &AppState,
    tenant: Option<pb::TenantContext>,
    store: Option<pb::StoreContext>,
) -> Result<Vec<pb::ProductAdmin>, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_id = StoreId::parse(&store_id)?;
    let tenant_id = TenantId::parse(&tenant_id)?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id,
               store_id::text as store_id,
               vendor_id::text as vendor_id,
               title,
               description,
               status,
               tax_rule_id::text as tax_rule_id,
               sale_start_at,
               sale_end_at,
               (SELECT category_id::text
                FROM product_category_links pc
                WHERE pc.product_id = products.id AND pc.is_primary = true
                LIMIT 1) as primary_category_id,
               (SELECT array_agg(category_id::text ORDER BY position)
                FROM product_category_links pc
                WHERE pc.product_id = products.id) as category_ids
        FROM products
        WHERE tenant_id = $1 AND store_id = $2
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(store_id.as_uuid())
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
            tax_rule_id: row.get::<Option<String>, _>("tax_rule_id").unwrap_or_default(),
            sale_start_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("sale_start_at")),
            sale_end_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("sale_end_at")),
            primary_category_id: row.get::<Option<String>, _>("primary_category_id").unwrap_or_default(),
            category_ids: row.get::<Option<Vec<String>>, _>("category_ids").unwrap_or_default(),
        })
        .collect())
}

pub async fn list_variants_admin(
    state: &AppState,
    tenant: Option<pb::TenantContext>,
    store: Option<pb::StoreContext>,
    product_id: String,
) -> Result<(Vec<pb::VariantAdmin>, Vec<pb::VariantAxis>), (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_id = StoreId::parse(&store_id)?;
    let product_id = ProductId::parse(&product_id)?;
    let axes_rows = sqlx::query(
        r#"
        SELECT id, name, position
        FROM product_variant_axes
        WHERE product_id = $1
        ORDER BY position ASC
        "#,
    )
    .bind(product_id.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    let variant_axes = axes_rows
        .iter()
        .map(|row| pb::VariantAxis {
            name: row.get::<String, _>("name"),
            position: row.get::<i32, _>("position") as u32,
        })
        .collect::<Vec<_>>();
    let rows = sqlx::query(
        r#"
        SELECT v.id,
               v.product_id,
               v.sku,
               v.jan_code,
               v.fulfillment_type,
               v.price_amount,
               v.price_currency,
               v.compare_at_amount,
               v.compare_at_currency,
               v.status,
               v.tax_rule_id
        FROM product_skus v
        JOIN products p ON p.id = v.product_id
        WHERE p.store_id = $1 AND v.product_id = $2
        ORDER BY v.created_at DESC
        "#,
    )
    .bind(store_id.as_uuid())
    .bind(product_id.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    let mut variants = Vec::with_capacity(rows.len());
    let mut variant_ids = Vec::with_capacity(rows.len());
    for row in rows {
        let variant_id: uuid::Uuid = row.get("id");
        let product_uuid: uuid::Uuid = row.get("product_id");
        variant_ids.push(variant_id);
        variants.push(pb::VariantAdmin {
            id: variant_id.to_string(),
            product_id: product_uuid.to_string(),
            sku: row.get("sku"),
            jan_code: row.get::<Option<String>, _>("jan_code").unwrap_or_default(),
            fulfillment_type: row.get("fulfillment_type"),
            price: Some(money_from_parts(
                row.get::<i64, _>("price_amount"),
                row.get::<String, _>("price_currency"),
            )),
            compare_at: row.get::<Option<i64>, _>("compare_at_amount").map(|amount| {
                money_from_parts(
                    amount,
                    row.get::<Option<String>, _>("compare_at_currency").unwrap_or_default(),
                )
            }),
            status: row.get("status"),
            tax_rule_id: row
                .get::<Option<uuid::Uuid>, _>("tax_rule_id")
                .map(|id| id.to_string())
                .unwrap_or_default(),
            axis_values: Vec::new(),
        });
    }

    if !variant_ids.is_empty() {
        let axis_rows = sqlx::query(
            r#"
            SELECT vav.variant_id,
                   ax.name as axis_name,
                   vav.value as axis_value
            FROM variant_axis_values vav
            JOIN product_variant_axes ax ON ax.id = vav.axis_id
            WHERE ax.product_id = $1
              AND vav.variant_id = ANY($2)
            ORDER BY ax.position ASC
            "#,
        )
        .bind(product_id.as_uuid())
        .bind(&variant_ids)
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?;

        let mut axis_map: std::collections::HashMap<uuid::Uuid, Vec<pb::VariantAxisValue>> =
            std::collections::HashMap::new();
        for row in axis_rows {
            let variant_id: uuid::Uuid = row.get("variant_id");
            axis_map.entry(variant_id).or_default().push(pb::VariantAxisValue {
                name: row.get::<String, _>("axis_name"),
                value: row.get::<String, _>("axis_value"),
            });
        }
        for variant in variants.iter_mut() {
            if let Ok(variant_uuid) = uuid::Uuid::parse_str(&variant.id)
                && let Some(values) = axis_map.remove(&variant_uuid)
            {
                variant.axis_values = values;
            }
        }
    }

    Ok((variants, variant_axes))
}

pub async fn list_skus_admin(
    state: &AppState,
    tenant: Option<pb::TenantContext>,
    store: Option<pb::StoreContext>,
    query: String,
) -> Result<Vec<pb::SkuAdmin>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_id = StoreId::parse(&store_id)?;
    let pattern = if query.trim().is_empty() {
        "".to_string()
    } else {
        format!("%{}%", query.trim())
    };

    let rows = sqlx::query(
        r#"
        SELECT v.id::text as id,
               v.sku,
               v.jan_code,
               v.product_id::text as product_id,
               p.title as product_title,
               v.fulfillment_type,
               v.price_amount,
               v.price_currency,
               v.status
        FROM product_skus v
        JOIN products p ON p.id = v.product_id
        WHERE p.store_id = $1
          AND ($2 = '' OR v.sku ILIKE $2 OR p.title ILIKE $2)
        ORDER BY v.created_at DESC
        LIMIT 50
        "#,
    )
    .bind(store_id.as_uuid())
    .bind(pattern)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    Ok(rows
        .into_iter()
        .map(|row| pb::SkuAdmin {
            id: row.get("id"),
            sku: row.get("sku"),
            product_id: row.get("product_id"),
            product_title: row.get("product_title"),
            fulfillment_type: row.get("fulfillment_type"),
            price: Some(money_from_parts(
                row.get::<i64, _>("price_amount"),
                row.get::<String, _>("price_currency"),
            )),
            status: row.get("status"),
            jan_code: row.get::<Option<String>, _>("jan_code").unwrap_or_default(),
        })
        .collect())
}

pub async fn create_product(
    state: &AppState,
    req: pb::CreateProductRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::ProductAdmin, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store.clone(), req.tenant.clone()).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let product_id = uuid::Uuid::new_v4();
    let tax_rule_id = nullable_uuid(req.tax_rule_id.clone());
    let sale_start_at = timestamp_to_chrono(req.sale_start_at.clone());
    let sale_end_at = timestamp_to_chrono(req.sale_end_at.clone());
    let status = ProductStatus::parse(&req.status)?.as_str().to_string();
    let (primary_category_id, category_ids) =
        normalize_category_ids(&req.primary_category_id, req.category_ids.clone())?;
    if let (Some(start), Some(end)) = (&sale_start_at, &sale_end_at)
        && start > end
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "sale_end_at must be later than sale_start_at".to_string(),
            }),
        ));
    }
    let mut tx = state.db.begin().await.map_err(db::error)?;
    ensure_category_ids_exist(&mut tx, &store_uuid.as_uuid(), &category_ids).await?;
    sqlx::query(
        r#"
        INSERT INTO products (
            id, tenant_id, store_id, vendor_id, title, description, status, tax_rule_id,
            sale_start_at, sale_end_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(product_id)
    .bind(tenant_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .bind(crate::shared::ids::nullable_uuid(req.vendor_id.clone()))
    .bind(&req.title)
    .bind(&req.description)
    .bind(&status)
    .bind(tax_rule_id)
    .bind(sale_start_at)
    .bind(sale_end_at)
    .execute(&mut *tx)
    .await
    .map_err(db::error)?;

    for (idx, category_id) in category_ids.iter().enumerate() {
        let category_uuid = parse_uuid(category_id, "category_id")?;
        let is_primary = !primary_category_id.is_empty() && category_id == &primary_category_id;
        sqlx::query(
            r#"
            INSERT INTO product_category_links (
                product_id, category_id, is_primary, position
            ) VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(product_id)
        .bind(category_uuid)
        .bind(is_primary)
        .bind((idx + 1) as i32)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    }

    if !req.variant_axes.is_empty() {
        for (idx, axis) in req.variant_axes.iter().enumerate() {
            let name = axis.name.trim();
            if name.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::InvalidArgument,
                        message: "variant_axes.name is required".to_string(),
                    }),
                ));
            }
            let position = if axis.position > 0 {
                axis.position as i32
            } else {
                (idx + 1) as i32
            };
            sqlx::query(
                r#"
                INSERT INTO product_variant_axes (id, product_id, name, position)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(uuid::Uuid::new_v4())
            .bind(product_id)
            .bind(name)
            .bind(position)
            .execute(&mut *tx)
            .await
            .map_err(db::error)?;
        }
    } else {
        let default_variant = req.default_variant.ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "default_variant is required when variant_axes is empty".to_string(),
                }),
            )
        })?;
        let default_sku = SkuCode::parse(&default_variant.sku)?;
        if default_variant.fulfillment_type.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "default_variant.fulfillment_type is required".to_string(),
                }),
            ));
        }
        if default_variant.status.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "default_variant.status is required".to_string(),
                }),
            ));
        }
        let (price_amount, price_currency) = money_to_parts(default_variant.price.clone())?;
        let (compare_amount, compare_currency) = money_to_parts_opt(default_variant.compare_at.clone())?;
        let fulfillment_type = FulfillmentType::parse(&default_variant.fulfillment_type)?
            .as_str()
            .to_string();
        let variant_status = VariantStatus::parse(&default_variant.status)?.as_str().to_string();
        sqlx::query(
            r#"
            INSERT INTO product_skus (
                id, product_id, sku, jan_code, fulfillment_type, price_amount, price_currency,
                compare_at_amount, compare_at_currency, status, tax_rule_id
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(product_id)
        .bind(default_sku.as_str())
        .bind(if default_variant.jan_code.is_empty() {
            None
        } else {
            Some(default_variant.jan_code.as_str())
        })
        .bind(&fulfillment_type)
        .bind(price_amount)
        .bind(&price_currency)
        .bind(compare_amount)
        .bind(compare_currency)
        .bind(&variant_status)
        .bind(tax_rule_id)
        .execute(&mut *tx)
        .await
        .map_err(db::error)?;
    }

    let product = pb::ProductAdmin {
        id: product_id.to_string(),
        vendor_id: req.vendor_id,
        title: req.title,
        description: req.description,
        status: status.clone(),
        updated_at: None,
        store_id: store_id.clone(),
        tax_rule_id: req.tax_rule_id,
        sale_start_at: chrono_to_timestamp(sale_start_at),
        sale_end_at: chrono_to_timestamp(sale_end_at),
        primary_category_id,
        category_ids: category_ids.clone(),
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            ProductAuditAction::Create.into(),
            Some("product"),
            Some(product.id.clone()),
            None,
            to_json_opt(Some(product.clone())),
            _actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db::error)?;

    let _ = reindex_product_by_id(state, &product.id).await;

    Ok(product)
}

fn normalize_category_ids(
    primary_category_id: &str,
    category_ids: Vec<String>,
) -> Result<(String, Vec<String>), (StatusCode, Json<ConnectError>)> {
    let primary = primary_category_id.trim();
    let mut ordered = category_ids
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    if ordered.is_empty() && primary.is_empty() {
        return Ok((String::new(), Vec::new()));
    }
    if primary.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "primary_category_id is required when category_ids are provided".to_string(),
            }),
        ));
    }
    if ordered.is_empty() {
        ordered.push(primary.to_string());
    }
    if !ordered.iter().any(|id| id == primary) {
        ordered.insert(0, primary.to_string());
    }
    let mut seen = std::collections::HashSet::new();
    ordered.retain(|id| seen.insert(id.clone()));
    Ok((primary.to_string(), ordered))
}

async fn ensure_category_ids_exist(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: &uuid::Uuid,
    category_ids: &[String],
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let parsed = category_ids
        .iter()
        .map(|id| parse_uuid(id, "category_id"))
        .collect::<Result<Vec<_>, _>>()?;
    if parsed.is_empty() {
        return Ok(());
    }
    let rows = sqlx::query("SELECT id FROM product_categories WHERE store_id = $1 AND id = ANY($2)")
        .bind(store_id)
        .bind(&parsed)
        .fetch_all(tx.as_mut())
        .await
        .map_err(db::error)?;
    if rows.len() != parsed.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "category_id is invalid".to_string(),
            }),
        ));
    }
    Ok(())
}

pub async fn update_product(
    state: &AppState,
    req: pb::UpdateProductRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::ProductAdmin, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, req.store.clone(), req.tenant.clone()).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let product_uuid = ProductId::parse(&req.product_id)?;
    let before = fetch_product_admin(state, &tenant_id, &store_id, &req.product_id)
        .await
        .ok();
    let tax_rule_id = nullable_uuid(req.tax_rule_id.clone());
    let sale_start_at = timestamp_to_chrono(req.sale_start_at.clone());
    let sale_end_at = timestamp_to_chrono(req.sale_end_at.clone());
    let status = ProductStatus::parse(&req.status)?.as_str().to_string();
    let (primary_category_id, category_ids) =
        normalize_category_ids(&req.primary_category_id, req.category_ids.clone())?;
    if let (Some(start), Some(end)) = (&sale_start_at, &sale_end_at)
        && start > end
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "sale_end_at must be later than sale_start_at".to_string(),
            }),
        ));
    }
    let mut tx = state.db.begin().await.map_err(db::error)?;
    ensure_category_ids_exist(&mut tx, &store_uuid.as_uuid(), &category_ids).await?;
    sqlx::query(
        r#"
        UPDATE products
        SET title = $1,
            description = $2,
            status = $3,
            tax_rule_id = $4,
            sale_start_at = $5,
            sale_end_at = $6,
            updated_at = now()
        WHERE id = $7 AND tenant_id = $8 AND store_id = $9
        "#,
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(&status)
    .bind(tax_rule_id)
    .bind(sale_start_at)
    .bind(sale_end_at)
    .bind(product_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .execute(tx.as_mut())
    .await
    .map_err(db::error)?;

    sqlx::query("DELETE FROM product_category_links WHERE product_id = $1")
        .bind(product_uuid.as_uuid())
        .execute(tx.as_mut())
        .await
        .map_err(db::error)?;

    for (idx, category_id) in category_ids.iter().enumerate() {
        let category_uuid = parse_uuid(category_id, "category_id")?;
        let is_primary = !primary_category_id.is_empty() && category_id == &primary_category_id;
        sqlx::query(
            r#"
            INSERT INTO product_category_links (
                product_id, category_id, is_primary, position
            ) VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(product_uuid.as_uuid())
        .bind(category_uuid)
        .bind(is_primary)
        .bind((idx + 1) as i32)
        .execute(tx.as_mut())
        .await
        .map_err(db::error)?;
    }

    if req.apply_tax_rule_to_variants {
        sqlx::query(
            r#"
            UPDATE product_skus
            SET tax_rule_id = $1, updated_at = now()
            WHERE product_id = $2
            "#,
        )
        .bind(tax_rule_id)
        .bind(product_uuid.as_uuid())
        .execute(tx.as_mut())
        .await
        .map_err(db::error)?;
    }

    let product = pb::ProductAdmin {
        id: req.product_id,
        vendor_id: String::new(),
        title: req.title,
        description: req.description,
        status: status.clone(),
        updated_at: None,
        store_id: store_id.clone(),
        tax_rule_id: req.tax_rule_id,
        sale_start_at: chrono_to_timestamp(sale_start_at),
        sale_end_at: chrono_to_timestamp(sale_end_at),
        primary_category_id: primary_category_id.clone(),
        category_ids: category_ids.clone(),
    };

    let mut after = product.clone();
    if let Ok(row) = sqlx::query(
        r#"
        SELECT id::text as id,
               store_id::text as store_id,
               vendor_id::text as vendor_id,
               title,
               description,
               status,
               tax_rule_id::text as tax_rule_id,
               sale_start_at,
               sale_end_at,
               (SELECT category_id::text
                FROM product_category_links pc
                WHERE pc.product_id = products.id AND pc.is_primary = true
                LIMIT 1) as primary_category_id,
               (SELECT array_agg(category_id::text ORDER BY position)
                FROM product_category_links pc
                WHERE pc.product_id = products.id) as category_ids
        FROM products
        WHERE tenant_id = $1 AND store_id = $2 AND id = $3
        "#,
    )
    .bind(tenant_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .bind(product_uuid.as_uuid())
    .fetch_one(tx.as_mut())
    .await
    {
        after = pb::ProductAdmin {
            id: row.get::<String, _>("id"),
            vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            updated_at: None,
            store_id: row.get::<String, _>("store_id"),
            tax_rule_id: row.get::<Option<String>, _>("tax_rule_id").unwrap_or_default(),
            sale_start_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("sale_start_at")),
            sale_end_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("sale_end_at")),
            primary_category_id: row.get::<Option<String>, _>("primary_category_id").unwrap_or_default(),
            category_ids: row.get::<Option<Vec<String>>, _>("category_ids").unwrap_or_default(),
        };
    }

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            ProductAuditAction::Update.into(),
            Some("product"),
            Some(product.id.clone()),
            to_json_opt(before),
            to_json_opt(Some(after.clone())),
            _actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db::error)?;

    let _ = reindex_product_by_id(state, &product.id).await;

    Ok(product)
}

pub async fn list_categories_admin(
    state: &AppState,
    store: Option<pb::StoreContext>,
    status: String,
) -> Result<Vec<pb::Category>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let rows = if status.trim().is_empty() {
        sqlx::query(
            r#"
            SELECT id::text as id,
                   store_id::text as store_id,
                   name,
                   slug,
                   description,
                   status,
                   parent_id::text as parent_id,
                   position,
                   created_at,
                   updated_at
            FROM product_categories
            WHERE store_id = $1
            ORDER BY parent_id NULLS FIRST, position ASC
            "#,
        )
        .bind(store_uuid.as_uuid())
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?
    } else {
        sqlx::query(
            r#"
            SELECT id::text as id,
                   store_id::text as store_id,
                   name,
                   slug,
                   description,
                   status,
                   parent_id::text as parent_id,
                   position,
                   created_at,
                   updated_at
            FROM product_categories
            WHERE store_id = $1 AND status = $2
            ORDER BY parent_id NULLS FIRST, position ASC
            "#,
        )
        .bind(store_uuid.as_uuid())
        .bind(status.trim())
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?
    };
    Ok(rows.into_iter().map(category_from_row).collect())
}

pub async fn create_category(
    state: &AppState,
    req: pb::CreateCategoryRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::Category, (StatusCode, Json<ConnectError>)> {
    let category = req.category.unwrap_or_default();
    let (store_id, tenant_id) = resolve_store_context(state, req.store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let name = category.name.trim();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "category.name is required".to_string(),
            }),
        ));
    }
    let slug = category.slug.trim();
    if slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "category.slug is required".to_string(),
            }),
        ));
    }
    let status = if category.status.trim().is_empty() {
        "active".to_string()
    } else {
        category.status.trim().to_string()
    };
    let parent_id = if category.parent_id.trim().is_empty() {
        None
    } else {
        Some(parse_uuid(&category.parent_id, "parent_id")?)
    };
    if let Some(parent_id) = parent_id {
        let exists = sqlx::query("SELECT id FROM product_categories WHERE store_id = $1 AND id = $2")
            .bind(store_uuid.as_uuid())
            .bind(parent_id)
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        if exists.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "parent_id is invalid".to_string(),
                }),
            ));
        }
    }
    let position = if category.position > 0 {
        category.position
    } else {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(position), 0) + 1 as next_pos FROM product_categories WHERE store_id = $1 AND parent_id IS NOT DISTINCT FROM $2",
        )
        .bind(store_uuid.as_uuid())
        .bind(parent_id)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
        row.get::<i32, _>("next_pos")
    };
    let row = sqlx::query(
        r#"
        INSERT INTO product_categories (
            tenant_id, store_id, parent_id, name, slug, description, status, position
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        RETURNING id::text as id,
                  store_id::text as store_id,
                  name,
                  slug,
                  description,
                  status,
                  parent_id::text as parent_id,
                  position,
                  created_at,
                  updated_at
        "#,
    )
    .bind(tenant_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .bind(parent_id)
    .bind(name)
    .bind(slug)
    .bind(if category.description.trim().is_empty() {
        None
    } else {
        Some(category.description.trim())
    })
    .bind(&status)
    .bind(position)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    Ok(category_from_row(row))
}

pub async fn update_category(
    state: &AppState,
    req: pb::UpdateCategoryRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::Category, (StatusCode, Json<ConnectError>)> {
    let category = req.category.unwrap_or_default();
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let category_uuid = parse_uuid(&req.category_id, "category_id")?;
    let name = category.name.trim();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "category.name is required".to_string(),
            }),
        ));
    }
    let slug = category.slug.trim();
    if slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "category.slug is required".to_string(),
            }),
        ));
    }
    let status = if category.status.trim().is_empty() {
        "active".to_string()
    } else {
        category.status.trim().to_string()
    };
    let parent_id = if category.parent_id.trim().is_empty() {
        None
    } else {
        Some(parse_uuid(&category.parent_id, "parent_id")?)
    };
    if let Some(parent_id) = parent_id {
        if parent_id == category_uuid {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "parent_id cannot be same as category_id".to_string(),
                }),
            ));
        }
        let exists = sqlx::query("SELECT id FROM product_categories WHERE store_id = $1 AND id = $2")
            .bind(store_uuid.as_uuid())
            .bind(parent_id)
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        if exists.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "parent_id is invalid".to_string(),
                }),
            ));
        }
    }
    let position = if category.position > 0 {
        Some(category.position)
    } else {
        None
    };
    let row = sqlx::query(
        r#"
        UPDATE product_categories
        SET name = $1,
            slug = $2,
            description = $3,
            status = $4,
            parent_id = $5,
            position = COALESCE($6, position),
            updated_at = now()
        WHERE id = $7 AND store_id = $8
        RETURNING id::text as id,
                  store_id::text as store_id,
                  name,
                  slug,
                  description,
                  status,
                  parent_id::text as parent_id,
                  position,
                  created_at,
                  updated_at
        "#,
    )
    .bind(name)
    .bind(slug)
    .bind(if category.description.trim().is_empty() {
        None
    } else {
        Some(category.description.trim())
    })
    .bind(&status)
    .bind(parent_id)
    .bind(position)
    .bind(category_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    Ok(category_from_row(row))
}

pub async fn delete_category(
    state: &AppState,
    req: pb::DeleteCategoryRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let category_uuid = parse_uuid(&req.category_id, "category_id")?;
    let children = sqlx::query("SELECT 1 FROM product_categories WHERE store_id = $1 AND parent_id = $2 LIMIT 1")
        .bind(store_uuid.as_uuid())
        .bind(category_uuid)
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?;
    if children.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "category has children".to_string(),
            }),
        ));
    }
    let linked = sqlx::query("SELECT 1 FROM product_category_links WHERE category_id = $1 LIMIT 1")
        .bind(category_uuid)
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?;
    if linked.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "category is used by products".to_string(),
            }),
        ));
    }
    let result = sqlx::query("DELETE FROM product_categories WHERE id = $1 AND store_id = $2")
        .bind(category_uuid)
        .bind(store_uuid.as_uuid())
        .execute(&state.db)
        .await
        .map_err(db::error)?;
    Ok(result.rows_affected() > 0)
}

pub async fn reorder_categories(
    state: &AppState,
    req: pb::ReorderCategoriesRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<Vec<pb::Category>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let ordered_ids = req
        .ordered_ids
        .iter()
        .map(|id| parse_uuid(id, "category_id"))
        .collect::<Result<Vec<_>, _>>()?;
    if ordered_ids.is_empty() {
        return Ok(Vec::new());
    }
    let parent_id = if req.parent_id.trim().is_empty() {
        None
    } else {
        Some(parse_uuid(&req.parent_id, "parent_id")?)
    };
    let rows = sqlx::query(
        "SELECT id FROM product_categories WHERE store_id = $1 AND parent_id IS NOT DISTINCT FROM $2 AND id = ANY($3)",
    )
    .bind(store_uuid.as_uuid())
    .bind(parent_id)
    .bind(&ordered_ids)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    if rows.len() != ordered_ids.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "ordered_ids contains invalid category".to_string(),
            }),
        ));
    }
    let mut tx = state.db.begin().await.map_err(db::error)?;
    for (idx, category_id) in ordered_ids.iter().enumerate() {
        sqlx::query("UPDATE product_categories SET position = $1, updated_at = now() WHERE id = $2 AND store_id = $3")
            .bind((idx + 1) as i32)
            .bind(category_id)
            .bind(store_uuid.as_uuid())
            .execute(tx.as_mut())
            .await
            .map_err(db::error)?;
    }
    tx.commit().await.map_err(db::error)?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id,
               store_id::text as store_id,
               name,
               slug,
               description,
               status,
               parent_id::text as parent_id,
               position,
               created_at,
               updated_at
        FROM product_categories
        WHERE store_id = $1 AND parent_id IS NOT DISTINCT FROM $2
        ORDER BY position ASC
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(parent_id)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    Ok(rows.into_iter().map(category_from_row).collect())
}

pub async fn list_category_products_admin(
    state: &AppState,
    store: Option<pb::StoreContext>,
    category_id: String,
) -> Result<Vec<pb::CategoryProductAdmin>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let category_uuid = parse_uuid(&category_id, "category_id")?;
    let exists = sqlx::query("SELECT 1 FROM product_categories WHERE id = $1 AND store_id = $2 LIMIT 1")
        .bind(category_uuid)
        .bind(store_uuid.as_uuid())
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?;
    if exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::NotFound,
                message: "category not found".to_string(),
            }),
        ));
    }
    let rows = sqlx::query(
        r#"
        SELECT p.id::text as product_id,
               p.title,
               p.status,
               pc.position
        FROM product_category_links pc
        JOIN products p ON p.id = pc.product_id
        WHERE pc.category_id = $1 AND p.store_id = $2
        ORDER BY pc.position ASC, p.created_at ASC
        "#,
    )
    .bind(category_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    Ok(rows.into_iter().map(category_product_from_row).collect())
}

pub async fn reorder_category_products(
    state: &AppState,
    req: pb::ReorderCategoryProductsRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<Vec<pb::CategoryProductAdmin>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, req.store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let category_uuid = parse_uuid(&req.category_id, "category_id")?;
    let ordered_ids = req
        .ordered_product_ids
        .iter()
        .map(|id| parse_uuid(id, "product_id"))
        .collect::<Result<Vec<_>, _>>()?;
    if ordered_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT pc.product_id::text as product_id
        FROM product_category_links pc
        JOIN products p ON p.id = pc.product_id
        WHERE pc.category_id = $1 AND p.store_id = $2
        ORDER BY pc.position ASC
        "#,
    )
    .bind(category_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    let existing_ids: Vec<String> = rows.into_iter().map(|row| row.get("product_id")).collect();
    if existing_ids.len() != ordered_ids.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "ordered_product_ids must include all products in the category".to_string(),
            }),
        ));
    }
    let mut sorted_existing = existing_ids.clone();
    sorted_existing.sort();
    let mut sorted_requested: Vec<String> = ordered_ids.iter().map(|id| id.to_string()).collect();
    sorted_requested.sort();
    if sorted_existing != sorted_requested {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "ordered_product_ids contains invalid product".to_string(),
            }),
        ));
    }

    let mut tx = state.db.begin().await.map_err(db::error)?;
    for (idx, product_id) in ordered_ids.iter().enumerate() {
        sqlx::query(
            r#"
            UPDATE product_category_links
            SET position = $1
            WHERE category_id = $2 AND product_id = $3
            "#,
        )
        .bind((idx + 1) as i32)
        .bind(category_uuid)
        .bind(product_id)
        .execute(tx.as_mut())
        .await
        .map_err(db::error)?;
    }
    tx.commit().await.map_err(db::error)?;

    let rows = sqlx::query(
        r#"
        SELECT p.id::text as product_id,
               p.title,
               p.status,
               pc.position
        FROM product_category_links pc
        JOIN products p ON p.id = pc.product_id
        WHERE pc.category_id = $1 AND p.store_id = $2
        ORDER BY pc.position ASC, p.created_at ASC
        "#,
    )
    .bind(category_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    Ok(rows.into_iter().map(category_product_from_row).collect())
}

fn category_from_row(row: sqlx::postgres::PgRow) -> pb::Category {
    pb::Category {
        id: row.get::<String, _>("id"),
        store_id: row.get::<String, _>("store_id"),
        name: row.get("name"),
        slug: row.get("slug"),
        description: row.get::<Option<String>, _>("description").unwrap_or_default(),
        status: row.get("status"),
        parent_id: row.get::<Option<String>, _>("parent_id").unwrap_or_default(),
        position: row.get::<i32, _>("position"),
        created_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")),
        updated_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at")),
    }
}

fn category_product_from_row(row: sqlx::postgres::PgRow) -> pb::CategoryProductAdmin {
    pb::CategoryProductAdmin {
        product_id: row.get::<String, _>("product_id"),
        title: row.get("title"),
        status: row.get("status"),
        position: row.get::<i32, _>("position"),
    }
}

fn metafield_definition_from_record(record: &metafields::MetafieldDefinitionRecord) -> pb::ProductMetafieldDefinition {
    pb::ProductMetafieldDefinition {
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

pub async fn list_product_metafield_definitions(
    state: &AppState,
) -> Result<Vec<pb::ProductMetafieldDefinition>, (StatusCode, Json<ConnectError>)> {
    let records = metafields::list_definitions(&state.db, METAFIELD_OWNER_TYPE_PRODUCT).await?;
    Ok(records
        .into_iter()
        .map(|record| metafield_definition_from_record(&record))
        .collect())
}

pub async fn create_product_metafield_definition(
    state: &AppState,
    input: pb::ProductMetafieldDefinitionInput,
) -> Result<pb::ProductMetafieldDefinition, (StatusCode, Json<ConnectError>)> {
    if input.namespace.is_empty() || input.key.is_empty() || input.name.is_empty() || input.value_type.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "definition required fields are missing".to_string(),
            }),
        ));
    }

    let validations_json = metafields::normalize_optional_json(input.validations_json)?;
    let visibility_json = metafields::normalize_optional_json(input.visibility_json)?;

    let record = metafields::create_definition(
        &state.db,
        METAFIELD_OWNER_TYPE_PRODUCT,
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
    .await?;

    Ok(metafield_definition_from_record(&record))
}

pub async fn update_product_metafield_definition(
    state: &AppState,
    definition_id: String,
    input: pb::ProductMetafieldDefinitionInput,
) -> Result<pb::ProductMetafieldDefinition, (StatusCode, Json<ConnectError>)> {
    if definition_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "definition_id is required".to_string(),
            }),
        ));
    }
    if input.namespace.is_empty() || input.key.is_empty() || input.name.is_empty() || input.value_type.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "definition required fields are missing".to_string(),
            }),
        ));
    }

    let definition_uuid = parse_uuid(&definition_id, "definition_id")?;
    let validations_json = metafields::normalize_optional_json(input.validations_json)?;
    let visibility_json = metafields::normalize_optional_json(input.visibility_json)?;

    let record = metafields::update_definition(
        &state.db,
        METAFIELD_OWNER_TYPE_PRODUCT,
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
    .await?;

    let Some(record) = record else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::NotFound,
                message: "metafield definition not found".to_string(),
            }),
        ));
    };

    Ok(metafield_definition_from_record(&record))
}

pub async fn list_product_metafield_values(
    state: &AppState,
    store: Option<pb::StoreContext>,
    product_id: String,
) -> Result<Vec<pb::ProductMetafieldValue>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let product_uuid = parse_uuid(&product_id, "product_id")?;

    let exists = sqlx::query(
        r#"
        SELECT 1
        FROM products
        WHERE id = $1 AND store_id = $2
        "#,
    )
    .bind(product_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    if exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::NotFound,
                message: "product not found".to_string(),
            }),
        ));
    }

    let records = metafields::list_values(&state.db, METAFIELD_OWNER_TYPE_PRODUCT, &product_uuid).await?;

    let values = records
        .into_iter()
        .map(|record| pb::ProductMetafieldValue {
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

pub async fn upsert_product_metafield_value(
    state: &AppState,
    store: Option<pb::StoreContext>,
    product_id: String,
    definition_id: String,
    value_json: String,
    _actor: Option<pb::ActorContext>,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, None).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let product_uuid = parse_uuid(&product_id, "product_id")?;
    let definition_uuid = parse_uuid(&definition_id, "definition_id")?;

    if value_json.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "value_json is required".to_string(),
            }),
        ));
    }

    serde_json::from_str::<serde_json::Value>(&value_json).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "value_json must be valid JSON".to_string(),
            }),
        )
    })?;

    let exists = sqlx::query(
        r#"
        SELECT 1
        FROM products
        WHERE id = $1 AND store_id = $2
        "#,
    )
    .bind(product_uuid)
    .bind(store_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    if exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::NotFound,
                message: "product not found".to_string(),
            }),
        ));
    }

    let definition = metafields::fetch_definition(&state.db, METAFIELD_OWNER_TYPE_PRODUCT, &definition_uuid).await?;

    let Some(definition) = definition else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::NotFound,
                message: "metafield definition not found".to_string(),
            }),
        ));
    };
    let value_type = definition.value_type.clone();
    let is_list = definition.is_list;
    if value_type == "bool" || value_type == "boolean" {
        let parsed = serde_json::from_str::<serde_json::Value>(&value_json).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "value_json must be a boolean".to_string(),
                }),
            )
        })?;
        let valid = if is_list {
            parsed.is_boolean()
                || parsed
                    .as_array()
                    .map(|items| items.iter().all(|item| item.is_boolean()))
                    .unwrap_or(false)
        } else {
            parsed.is_boolean()
        };
        if !valid {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "value_json must be a boolean".to_string(),
                }),
            ));
        }
    }

    let _definition = metafield_definition_from_record(&definition);

    metafields::upsert_value(&state.db, &definition_uuid, &product_uuid, &value_json).await?;

    Ok(())
}

pub async fn create_variant(
    state: &AppState,
    req: pb::CreateVariantRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::VariantAdmin, (StatusCode, Json<ConnectError>)> {
    let tenant_id = tenant_id_for_product(state, &req.product_id).await?;
    let store_id = store_id_for_tenant(state, &tenant_id).await?;
    if let Some(tenant) = req.tenant.as_ref().and_then(|t| {
        if t.tenant_id.is_empty() {
            None
        } else {
            Some(t.tenant_id.as_str())
        }
    }) && tenant != tenant_id
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "tenant does not match product".to_string(),
            }),
        ));
    }
    let variant_id = uuid::Uuid::new_v4();
    let (price_amount, price_currency) = money_to_parts(req.price.clone())?;
    let (compare_amount, compare_currency) = money_to_parts_opt(req.compare_at.clone())?;
    let fulfillment_type = FulfillmentType::parse(&req.fulfillment_type)?.as_str().to_string();
    let status = VariantStatus::parse(&req.status)?.as_str().to_string();
    let sku = SkuCode::parse(&req.sku)?;
    let mut tx = state.db.begin().await.map_err(db::error)?;
    let axes_rows = sqlx::query(
        r#"
        SELECT id, name, position
        FROM product_variant_axes
        WHERE product_id = $1
        ORDER BY position ASC
        "#,
    )
    .bind(parse_uuid(&req.product_id, "product_id")?)
    .fetch_all(tx.as_mut())
    .await
    .map_err(db::error)?;
    let mut axis_value_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for axis_value in req.axis_values.iter() {
        let name = axis_value.name.trim().to_lowercase();
        let value = axis_value.value.trim();
        if name.is_empty() || value.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "axis_values.name and axis_values.value are required".to_string(),
                }),
            ));
        }
        if axis_value_map.insert(name, value.to_string()).is_some() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "axis_values contains duplicate axis names".to_string(),
                }),
            ));
        }
    }
    if axes_rows.is_empty() && !req.axis_values.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "axis_values cannot be set when variant axes are not defined".to_string(),
            }),
        ));
    }
    let mut axis_values_for_response = Vec::new();
    if !axes_rows.is_empty() {
        for row in axes_rows.iter() {
            let axis_name = row.get::<String, _>("name");
            let key = axis_name.trim().to_lowercase();
            let value = axis_value_map.get(&key).cloned().unwrap_or_default();
            if value.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::InvalidArgument,
                        message: format!("axis value is required for {}", axis_name),
                    }),
                ));
            }
            axis_values_for_response.push(pb::VariantAxisValue {
                name: axis_name.clone(),
                value: value.clone(),
            });
        }
    }
    let product_tax_rule_id = sqlx::query("SELECT tax_rule_id FROM products WHERE id = $1")
        .bind(parse_uuid(&req.product_id, "product_id")?)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(db::error)?
        .and_then(|row| row.get::<Option<uuid::Uuid>, _>("tax_rule_id"));
    sqlx::query(
        r#"
        INSERT INTO product_skus (
            id, product_id, sku, jan_code, fulfillment_type, price_amount, price_currency,
            compare_at_amount, compare_at_currency, status, tax_rule_id
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        "#,
    )
    .bind(variant_id)
    .bind(parse_uuid(&req.product_id, "product_id")?)
    .bind(sku.as_str())
    .bind(if req.jan_code.is_empty() {
        None
    } else {
        Some(req.jan_code.as_str())
    })
    .bind(&fulfillment_type)
    .bind(price_amount)
    .bind(&price_currency)
    .bind(compare_amount)
    .bind(compare_currency)
    .bind(&status)
    .bind(product_tax_rule_id)
    .execute(tx.as_mut())
    .await
    .map_err(db::error)?;

    for row in axes_rows.iter() {
        let axis_id: uuid::Uuid = row.get("id");
        let axis_name = row.get::<String, _>("name");
        let key = axis_name.trim().to_lowercase();
        if let Some(value) = axis_value_map.get(&key) {
            sqlx::query(
                r#"
                INSERT INTO variant_axis_values (id, variant_id, axis_id, value)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(uuid::Uuid::new_v4())
            .bind(variant_id)
            .bind(axis_id)
            .bind(value)
            .execute(tx.as_mut())
            .await
            .map_err(db::error)?;
        }
    }

    let variant = pb::VariantAdmin {
        id: variant_id.to_string(),
        product_id: req.product_id,
        sku: req.sku,
        fulfillment_type,
        price: req.price,
        compare_at: req.compare_at,
        status,
        tax_rule_id: product_tax_rule_id.map(|id| id.to_string()).unwrap_or_default(),
        axis_values: axis_values_for_response,
        jan_code: if req.jan_code.is_empty() {
            String::new()
        } else {
            req.jan_code
        },
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            VariantAuditAction::Create.into(),
            Some("variant"),
            Some(variant.id.clone()),
            None,
            to_json_opt(Some(variant.clone())),
            _actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db::error)?;

    let _ = reindex_product_by_id(state, &variant.product_id).await;

    Ok(variant)
}

pub async fn update_variant(
    state: &AppState,
    req: pb::UpdateVariantRequest,
    _actor: Option<pb::ActorContext>,
) -> Result<pb::VariantAdmin, (StatusCode, Json<ConnectError>)> {
    let tenant_id = tenant_id_for_variant(state, &req.variant_id).await?;
    let store_id = store_id_for_tenant(state, &tenant_id).await?;
    if let Some(tenant) = req.tenant.as_ref().and_then(|t| {
        if t.tenant_id.is_empty() {
            None
        } else {
            Some(t.tenant_id.as_str())
        }
    }) && tenant != tenant_id
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "tenant does not match variant".to_string(),
            }),
        ));
    }
    let (price_amount, price_currency) = money_to_parts(req.price.clone())?;
    let (compare_amount, compare_currency) = money_to_parts_opt(req.compare_at.clone())?;
    let fulfillment_type = if req.fulfillment_type.is_empty() {
        None
    } else {
        Some(FulfillmentType::parse(&req.fulfillment_type)?.as_str().to_string())
    };
    let status = VariantStatus::parse(&req.status)?.as_str().to_string();
    let axes_rows = sqlx::query(
        r#"
        SELECT ax.id, ax.name
        FROM product_variant_axes ax
        JOIN product_skus v ON v.product_id = ax.product_id
        WHERE v.id = $1
        ORDER BY ax.position ASC
        "#,
    )
    .bind(parse_uuid(&req.variant_id, "variant_id")?)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    let mut axis_value_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for axis_value in req.axis_values.iter() {
        let name = axis_value.name.trim().to_lowercase();
        let value = axis_value.value.trim();
        if name.is_empty() || value.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "axis_values.name and axis_values.value are required".to_string(),
                }),
            ));
        }
        if axis_value_map.insert(name, value.to_string()).is_some() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "axis_values contains duplicate axis names".to_string(),
                }),
            ));
        }
    }
    if axes_rows.is_empty() && !req.axis_values.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "axis_values cannot be set when variant axes are not defined".to_string(),
            }),
        ));
    }
    let mut tx = state.db.begin().await.map_err(db::error)?;
    sqlx::query(
        r#"
        UPDATE product_skus
        SET price_amount = $1, price_currency = $2,
            compare_at_amount = $3, compare_at_currency = $4,
            status = $5,
            fulfillment_type = COALESCE($6, fulfillment_type),
            jan_code = NULLIF($7, ''),
            updated_at = now()
        WHERE id = $8
        "#,
    )
    .bind(price_amount)
    .bind(&price_currency)
    .bind(compare_amount)
    .bind(compare_currency)
    .bind(&status)
    .bind(fulfillment_type.as_deref())
    .bind(req.jan_code.as_str())
    .bind(parse_uuid(&req.variant_id, "variant_id")?)
    .execute(tx.as_mut())
    .await
    .map_err(db::error)?;

    let mut axis_values_for_response = Vec::new();
    if !req.axis_values.is_empty() {
        for row in axes_rows.iter() {
            let axis_name = row.get::<String, _>("name");
            let key = axis_name.trim().to_lowercase();
            let value = axis_value_map.get(&key).cloned().unwrap_or_default();
            if value.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::InvalidArgument,
                        message: format!("axis value is required for {}", axis_name),
                    }),
                ));
            }
            axis_values_for_response.push(pb::VariantAxisValue {
                name: axis_name.clone(),
                value: value.clone(),
            });
        }
        sqlx::query("DELETE FROM variant_axis_values WHERE variant_id = $1")
            .bind(parse_uuid(&req.variant_id, "variant_id")?)
            .execute(tx.as_mut())
            .await
            .map_err(db::error)?;
        for row in axes_rows.iter() {
            let axis_id: uuid::Uuid = row.get("id");
            let axis_name = row.get::<String, _>("name");
            let key = axis_name.trim().to_lowercase();
            if let Some(value) = axis_value_map.get(&key) {
                sqlx::query(
                    r#"
                    INSERT INTO variant_axis_values (id, variant_id, axis_id, value)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(uuid::Uuid::new_v4())
                .bind(parse_uuid(&req.variant_id, "variant_id")?)
                .bind(axis_id)
                .bind(value)
                .execute(tx.as_mut())
                .await
                .map_err(db::error)?;
            }
        }
    } else if !axes_rows.is_empty() {
        let values_rows = sqlx::query(
            r#"
            SELECT ax.name as axis_name, vav.value as axis_value
            FROM variant_axis_values vav
            JOIN product_variant_axes ax ON ax.id = vav.axis_id
            WHERE vav.variant_id = $1
            ORDER BY ax.position ASC
            "#,
        )
        .bind(parse_uuid(&req.variant_id, "variant_id")?)
        .fetch_all(tx.as_mut())
        .await
        .map_err(db::error)?;
        axis_values_for_response = values_rows
            .iter()
            .map(|row| pb::VariantAxisValue {
                name: row.get::<String, _>("axis_name"),
                value: row.get::<String, _>("axis_value"),
            })
            .collect();
    }

    let row = sqlx::query(
        r#"
        SELECT id::text as id,
               product_id::text as product_id,
               sku,
               jan_code,
               fulfillment_type,
               price_amount,
               price_currency,
               compare_at_amount,
               compare_at_currency,
               status,
               tax_rule_id::text as tax_rule_id
        FROM product_skus
        WHERE id = $1
        "#,
    )
    .bind(parse_uuid(&req.variant_id, "variant_id")?)
    .fetch_one(tx.as_mut())
    .await
    .map_err(db::error)?;

    let variant = pb::VariantAdmin {
        id: row.get("id"),
        product_id: row.get("product_id"),
        sku: row.get("sku"),
        jan_code: row.get::<Option<String>, _>("jan_code").unwrap_or_default(),
        fulfillment_type: row.get("fulfillment_type"),
        price: Some(money_from_parts(
            row.get::<i64, _>("price_amount"),
            row.get::<String, _>("price_currency"),
        )),
        compare_at: row.get::<Option<i64>, _>("compare_at_amount").map(|amount| {
            money_from_parts(
                amount,
                row.get::<Option<String>, _>("compare_at_currency").unwrap_or_default(),
            )
        }),
        status: row.get("status"),
        tax_rule_id: row.get::<Option<String>, _>("tax_rule_id").unwrap_or_default(),
        axis_values: axis_values_for_response,
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            VariantAuditAction::Update.into(),
            Some("variant"),
            Some(variant.id.clone()),
            None,
            to_json_opt(Some(variant.clone())),
            _actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db::error)?;

    if let Ok(row) = sqlx::query("SELECT product_id::text as product_id FROM product_skus WHERE id = $1")
        .bind(parse_uuid(&variant.id, "variant_id")?)
        .fetch_one(&state.db)
        .await
    {
        let product_id: String = row.get("product_id");
        let _ = reindex_product_by_id(state, &product_id).await;
    }

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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "location_id is required".to_string(),
            }),
        ));
    }
    let (store_id, tenant_id) = resolve_store_context(state, req.store.clone(), req.tenant.clone()).await?;
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    ensure_variant_belongs_to_store(state, &req.variant_id, &store_id).await?;
    ensure_location_belongs_to_store(state, &req.location_id, &store_id).await?;
    let mut tx = state.db.begin().await.map_err(db::error)?;
    sqlx::query(
        r#"
        INSERT INTO inventory_stocks (tenant_id, store_id, location_id, variant_id, stock, reserved)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (variant_id, location_id)
        DO UPDATE SET stock = EXCLUDED.stock, reserved = EXCLUDED.reserved, updated_at = now()
    "#,
    )
    .bind(tenant_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .bind(parse_uuid(&req.location_id, "location_id")?)
    .bind(parse_uuid(&req.variant_id, "variant_id")?)
    .bind(req.stock)
    .bind(req.reserved)
    .execute(tx.as_mut())
    .await
    .map_err(db::error)?;

    let inventory = pb::InventoryAdmin {
        variant_id: req.variant_id,
        location_id: req.location_id,
        stock: req.stock,
        reserved: req.reserved,
        updated_at: None,
    };

    audit::record_tx(
        &mut tx,
        audit_input(
            Some(store_id.clone()),
            InventoryAuditAction::Set.into(),
            Some("inventory"),
            Some(inventory.variant_id.clone()),
            None,
            to_json_opt(Some(inventory.clone())),
            _actor,
        ),
    )
    .await?;

    tx.commit().await.map_err(db::error)?;

    if let Ok(row) = sqlx::query(
        r#"
        SELECT p.id::text as id
        FROM products p
        JOIN product_skus v ON v.product_id = p.id
        WHERE v.id = $1
        "#,
    )
    .bind(parse_uuid(&inventory.variant_id, "variant_id")?)
    .fetch_one(&state.db)
    .await
    {
        let product_id: String = row.get("id");
        let _ = reindex_product_by_id(state, &product_id).await;
    }

    Ok(inventory)
}

async fn fetch_product_admin(
    state: &AppState,
    tenant_id: &str,
    store_id: &str,
    product_id: &str,
) -> Result<pb::ProductAdmin, (StatusCode, Json<ConnectError>)> {
    let tenant_uuid = TenantId::parse(tenant_id)?;
    let store_uuid = StoreId::parse(store_id)?;
    let product_uuid = ProductId::parse(product_id)?;
    let row = sqlx::query(
        r#"
        SELECT id::text as id,
               store_id::text as store_id,
               vendor_id::text as vendor_id,
               title,
               description,
               status,
               tax_rule_id::text as tax_rule_id,
               sale_start_at,
               sale_end_at,
               (SELECT category_id::text
                FROM product_category_links pc
                WHERE pc.product_id = products.id AND pc.is_primary = true
                LIMIT 1) as primary_category_id,
               (SELECT array_agg(category_id::text ORDER BY position)
                FROM product_category_links pc
                WHERE pc.product_id = products.id) as category_ids
        FROM products
        WHERE tenant_id = $1 AND store_id = $2 AND id = $3
        "#,
    )
    .bind(tenant_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .bind(product_uuid.as_uuid())
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
        tax_rule_id: row.get::<Option<String>, _>("tax_rule_id").unwrap_or_default(),
        sale_start_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("sale_start_at")),
        sale_end_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("sale_end_at")),
        primary_category_id: row.get::<Option<String>, _>("primary_category_id").unwrap_or_default(),
        category_ids: row.get::<Option<Vec<String>>, _>("category_ids").unwrap_or_default(),
    })
}

async fn tenant_id_for_product(state: &AppState, product_id: &str) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM products WHERE id = $1")
        .bind(parse_uuid(product_id, "product_id")?)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
    Ok(row.get("tenant_id"))
}

async fn tenant_id_for_variant(state: &AppState, variant_id: &str) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT p.tenant_id::text as tenant_id
        FROM products p
        JOIN product_skus v ON v.product_id = p.id
        WHERE v.id = $1
        "#,
    )
    .bind(parse_uuid(variant_id, "variant_id")?)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;
    Ok(row.get("tenant_id"))
}

async fn store_id_for_tenant(state: &AppState, tenant_id: &str) -> Result<String, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query("SELECT id::text as id FROM stores WHERE tenant_id = $1 ORDER BY created_at ASC LIMIT 1")
        .bind(parse_uuid(tenant_id, "tenant_id")?)
        .fetch_optional(&state.db)
        .await
        .map_err(db::error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "store not found for tenant".to_string(),
            }),
        ));
    };
    Ok(row.get("id"))
}

async fn resolve_store_context(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> Result<(String, String), (StatusCode, Json<ConnectError>)> {
    if let Some(ctx) = request_context::current() {
        if let Some(auth_store) = ctx.store_id.as_deref()
            && let Some(store_id) = store.as_ref().and_then(|s| {
                if s.store_id.is_empty() {
                    None
                } else {
                    Some(s.store_id.as_str())
                }
            })
            && store_id != auth_store
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "store_id does not match token".to_string(),
                }),
            ));
        }
        if let Some(auth_tenant) = ctx.tenant_id.as_deref()
            && let Some(tenant_id) = tenant.as_ref().and_then(|t| {
                if t.tenant_id.is_empty() {
                    None
                } else {
                    Some(t.tenant_id.as_str())
                }
            })
            && tenant_id != auth_tenant
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "tenant_id does not match token".to_string(),
                }),
            ));
        }
    }

    if let Some(store_id) = store.as_ref().and_then(|s| {
        if s.store_id.is_empty() {
            None
        } else {
            Some(s.store_id.as_str())
        }
    }) {
        let store_uuid = StoreId::parse(store_id)?;
        let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
            .bind(store_uuid.as_uuid())
            .fetch_one(&state.db)
            .await
            .map_err(db::error)?;
        let tenant_id: String = row.get("tenant_id");
        return Ok((store_id.to_string(), tenant_id));
    }
    if let Some(store_code) = store.as_ref().and_then(|s| {
        if s.store_code.is_empty() {
            None
        } else {
            Some(s.store_code.as_str())
        }
    }) {
        let row = sqlx::query("SELECT id::text as id, tenant_id::text as tenant_id FROM stores WHERE code = $1")
            .bind(store_code)
            .fetch_optional(&state.db)
            .await
            .map_err(db::error)?;
        let Some(row) = row else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "store_code not found".to_string(),
                }),
            ));
        };
        let store_id: String = row.get("id");
        let tenant_id: String = row.get("tenant_id");
        if let Some(ctx) = request_context::current()
            && let Some(auth_store) = ctx.store_id
            && auth_store != store_id
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::PermissionDenied,
                    message: "store_code does not match token".to_string(),
                }),
            ));
        }
        return Ok((store_id, tenant_id));
    }
    if let Some(tenant_id) = tenant.and_then(|t| {
        if t.tenant_id.is_empty() {
            None
        } else {
            Some(t.tenant_id)
        }
    }) {
        let store_id = store_id_for_tenant(state, &tenant_id).await?;
        return Ok((store_id, tenant_id));
    }
    if let Some(ctx) = request_context::current() {
        if let Some(store_id) = ctx.store_id {
            let store_uuid = StoreId::parse(&store_id)?;
            let row = sqlx::query("SELECT tenant_id::text as tenant_id FROM stores WHERE id = $1")
                .bind(store_uuid.as_uuid())
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
            code: crate::rpc::json::ErrorCode::InvalidArgument,
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
        FROM product_skus v
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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
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
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "location does not belong to store".to_string(),
            }),
        ));
    }
    Ok(())
}

async fn reindex_product_by_id(state: &AppState, product_id: &str) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT id::text as id, tenant_id::text as tenant_id, store_id::text as store_id,
               vendor_id::text as vendor_id,
               title, description, status
        FROM products
        WHERE id = $1
        "#,
    )
    .bind(parse_uuid(product_id, "product_id")?)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    let category_rows = sqlx::query(
        r#"
        SELECT category_id::text as category_id, is_primary, position
        FROM product_category_links
        WHERE product_id = $1
        ORDER BY position ASC
        "#,
    )
    .bind(parse_uuid(product_id, "product_id")?)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    let mut category_ids = Vec::new();
    let mut primary_category_id = String::new();
    for row in category_rows {
        let category_id = row.get::<String, _>("category_id");
        if row.get::<bool, _>("is_primary") && primary_category_id.is_empty() {
            primary_category_id = category_id.clone();
        }
        category_ids.push(category_id);
    }

    let sku_rows = sqlx::query(
        r#"
        SELECT sku
        FROM product_skus
        WHERE product_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(parse_uuid(product_id, "product_id")?)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;
    let sku_codes = sku_rows
        .into_iter()
        .map(|row| row.get::<String, _>("sku"))
        .collect::<Vec<_>>();

    state
        .search
        .upsert_products(&[crate::infrastructure::search::SearchProduct {
            id: row.get::<String, _>("id"),
            tenant_id: row.get::<String, _>("tenant_id"),
            store_id: row.get::<String, _>("store_id"),
            vendor_id: row.get::<Option<String>, _>("vendor_id").unwrap_or_default(),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            primary_category_id,
            category_ids,
            sku_codes,
        }])
        .await?;
    Ok(())
}
