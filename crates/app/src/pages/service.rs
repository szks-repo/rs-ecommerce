use axum::{Json, http::StatusCode};
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::{
    AppState,
    infrastructure::db,
    pages::status::PageStatus,
    pb::pb,
    rpc::json::{ConnectError, invalid_argument, not_found},
    shared::{
        ids::{StoreId, TenantId, parse_uuid},
        time::{chrono_to_timestamp, timestamp_to_chrono},
    },
};

type PageResult<T> = Result<T, (StatusCode, Json<ConnectError>)>;

pub async fn list_pages(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    page: Option<pb::PageInfo>,
) -> PageResult<(Vec<pb::PageSummary>, pb::PageResult)> {
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let (limit, offset) = page_params(page);
    let rows = sqlx::query(
        r#"
        SELECT id::text as id,
               title,
               slug,
               status,
               publish_start_at,
               publish_end_at,
               updated_at
          FROM pages
         WHERE store_id = $1 AND tenant_id = $2
         ORDER BY updated_at DESC, id DESC
         LIMIT $3 OFFSET $4
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    let pages = rows
        .into_iter()
        .map(|row| pb::PageSummary {
            id: row.get("id"),
            title: row.get("title"),
            slug: row.get("slug"),
            status: row.get("status"),
            publish_start_at: chrono_to_timestamp(row.get::<Option<DateTime<Utc>>, _>("publish_start_at")),
            publish_end_at: chrono_to_timestamp(row.get::<Option<DateTime<Utc>>, _>("publish_end_at")),
            updated_at: chrono_to_timestamp(Some(row.get::<DateTime<Utc>, _>("updated_at"))),
        })
        .collect::<Vec<_>>();

    let mut next_page_token = String::new();
    if (pages.len() as i64) == limit {
        next_page_token = (offset + limit).to_string();
    }

    Ok((pages, pb::PageResult { next_page_token }))
}

pub async fn get_page(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    page_id: String,
) -> PageResult<pb::PageAdmin> {
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let page_uuid = parse_uuid(&page_id, "page_id")?;
    let row = sqlx::query(
        r#"
        SELECT id::text as id,
               store_id::text as store_id,
               title,
               slug,
               body,
               body_format,
               status,
               publish_start_at,
               publish_end_at,
               seo_title,
               seo_description,
               created_at,
               updated_at
          FROM pages
         WHERE store_id = $1 AND tenant_id = $2 AND id = $3
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .bind(page_uuid)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    let Some(row) = row else {
        return Err(not_found("page not found"));
    };
    Ok(page_admin_from_row(&row))
}

pub async fn create_page(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    input: pb::PageInput,
) -> PageResult<pb::PageAdmin> {
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let input = validate_page_input(input)?;
    let row = sqlx::query(
        r#"
        INSERT INTO pages (
            tenant_id,
            store_id,
            title,
            slug,
            body,
            body_format,
            status,
            publish_start_at,
            publish_end_at,
            seo_title,
            seo_description
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        RETURNING id::text as id,
                  store_id::text as store_id,
                  title,
                  slug,
                  body,
                  body_format,
                  status,
                  publish_start_at,
                  publish_end_at,
                  seo_title,
                  seo_description,
                  created_at,
                  updated_at
        "#,
    )
    .bind(tenant_uuid.as_uuid())
    .bind(store_uuid.as_uuid())
    .bind(&input.title)
    .bind(&input.slug)
    .bind(&input.body)
    .bind(&input.body_format)
    .bind(input.status.as_str())
    .bind(input.publish_start_at)
    .bind(input.publish_end_at)
    .bind(&input.seo_title)
    .bind(&input.seo_description)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    Ok(page_admin_from_row(&row))
}

pub async fn update_page(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    page_id: String,
    input: pb::PageInput,
) -> PageResult<pb::PageAdmin> {
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let page_uuid = parse_uuid(&page_id, "page_id")?;
    let input = validate_page_input(input)?;
    let row = sqlx::query(
        r#"
        UPDATE pages
           SET title = $1,
               slug = $2,
               body = $3,
               body_format = $4,
               status = $5,
               publish_start_at = $6,
               publish_end_at = $7,
               seo_title = $8,
               seo_description = $9,
               updated_at = now()
         WHERE id = $10 AND store_id = $11 AND tenant_id = $12
         RETURNING id::text as id,
                   store_id::text as store_id,
                   title,
                   slug,
                   body,
                   body_format,
                   status,
                   publish_start_at,
                   publish_end_at,
                   seo_title,
                   seo_description,
                   created_at,
                   updated_at
        "#,
    )
    .bind(&input.title)
    .bind(&input.slug)
    .bind(&input.body)
    .bind(&input.body_format)
    .bind(input.status.as_str())
    .bind(input.publish_start_at)
    .bind(input.publish_end_at)
    .bind(&input.seo_title)
    .bind(&input.seo_description)
    .bind(page_uuid)
    .bind(store_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    let Some(row) = row else {
        return Err(not_found("page not found"));
    };
    Ok(page_admin_from_row(&row))
}

pub async fn delete_page(state: &AppState, store_id: String, tenant_id: String, page_id: String) -> PageResult<bool> {
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let page_uuid = parse_uuid(&page_id, "page_id")?;
    let rows = sqlx::query("DELETE FROM pages WHERE id = $1 AND store_id = $2 AND tenant_id = $3")
        .bind(page_uuid)
        .bind(store_uuid.as_uuid())
        .bind(tenant_uuid.as_uuid())
        .execute(&state.db)
        .await
        .map_err(db::error)?;
    Ok(rows.rows_affected() > 0)
}

pub async fn get_page_by_slug(
    state: &AppState,
    store_id: String,
    tenant_id: String,
    slug: String,
) -> PageResult<pb::StorefrontPage> {
    let store_uuid = StoreId::parse(&store_id)?;
    let tenant_uuid = TenantId::parse(&tenant_id)?;
    let slug = normalize_slug(&slug)?;
    let row = sqlx::query(
        r#"
        SELECT id::text as id,
               title,
               slug,
               body,
               body_format,
               publish_start_at,
               publish_end_at,
               seo_title,
               seo_description,
               updated_at
          FROM pages
         WHERE store_id = $1
           AND tenant_id = $2
           AND slug = $3
           AND status = 'published'
           AND (publish_start_at IS NULL OR publish_start_at <= now())
           AND (publish_end_at IS NULL OR publish_end_at > now())
        "#,
    )
    .bind(store_uuid.as_uuid())
    .bind(tenant_uuid.as_uuid())
    .bind(slug)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    let Some(row) = row else {
        return Err(not_found("page not found"));
    };
    Ok(pb::StorefrontPage {
        id: row.get("id"),
        title: row.get("title"),
        slug: row.get("slug"),
        body: row.get("body"),
        body_format: row.get("body_format"),
        publish_start_at: chrono_to_timestamp(row.get::<Option<DateTime<Utc>>, _>("publish_start_at")),
        publish_end_at: chrono_to_timestamp(row.get::<Option<DateTime<Utc>>, _>("publish_end_at")),
        seo_title: row.get::<Option<String>, _>("seo_title").unwrap_or_default(),
        seo_description: row.get::<Option<String>, _>("seo_description").unwrap_or_default(),
        updated_at: chrono_to_timestamp(Some(row.get::<DateTime<Utc>, _>("updated_at"))),
    })
}

fn page_admin_from_row(row: &sqlx::postgres::PgRow) -> pb::PageAdmin {
    pb::PageAdmin {
        id: row.get("id"),
        store_id: row.get("store_id"),
        title: row.get("title"),
        slug: row.get("slug"),
        body: row.get("body"),
        body_format: row.get("body_format"),
        status: row.get("status"),
        publish_start_at: chrono_to_timestamp(row.get::<Option<DateTime<Utc>>, _>("publish_start_at")),
        publish_end_at: chrono_to_timestamp(row.get::<Option<DateTime<Utc>>, _>("publish_end_at")),
        seo_title: row.get::<Option<String>, _>("seo_title").unwrap_or_default(),
        seo_description: row.get::<Option<String>, _>("seo_description").unwrap_or_default(),
        created_at: chrono_to_timestamp(Some(row.get::<DateTime<Utc>, _>("created_at"))),
        updated_at: chrono_to_timestamp(Some(row.get::<DateTime<Utc>, _>("updated_at"))),
    }
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

fn validate_page_input(input: pb::PageInput) -> PageResult<ValidatedPageInput> {
    let title = require_text(&input.title, "title")?;
    let slug = normalize_slug(&input.slug)?;
    let body = input.body.trim().to_string();
    let body_format = normalize_body_format(&input.body_format)?;
    let status = normalize_status(&input.status)?;
    let publish_start_at = timestamp_to_chrono(input.publish_start_at);
    let publish_end_at = timestamp_to_chrono(input.publish_end_at);
    if let (Some(start), Some(end)) = (publish_start_at, publish_end_at) {
        if end < start {
            return Err(invalid_argument("publish_end_at must be after publish_start_at"));
        }
    }
    let seo_title = normalize_optional(&input.seo_title);
    let seo_description = normalize_optional(&input.seo_description);
    Ok(ValidatedPageInput {
        title,
        slug,
        body,
        body_format,
        status,
        publish_start_at,
        publish_end_at,
        seo_title,
        seo_description,
    })
}

struct ValidatedPageInput {
    title: String,
    slug: String,
    body: String,
    body_format: String,
    status: PageStatus,
    publish_start_at: Option<DateTime<Utc>>,
    publish_end_at: Option<DateTime<Utc>>,
    seo_title: Option<String>,
    seo_description: Option<String>,
}

fn require_text(value: &str, field: &str) -> PageResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(invalid_argument(format!("{} is required", field)));
    }
    Ok(trimmed.to_string())
}

fn normalize_slug(value: &str) -> PageResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(invalid_argument("slug is required"));
    }
    if trimmed.len() > 255 {
        return Err(invalid_argument("slug must be 255 chars or less"));
    }
    Ok(trimmed.to_string())
}

fn normalize_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_body_format(value: &str) -> PageResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok("markdown".to_string());
    }
    if trimmed != "markdown" {
        return Err(invalid_argument("body_format must be markdown"));
    }
    Ok(trimmed.to_string())
}

fn normalize_status(value: &str) -> PageResult<PageStatus> {
    PageStatus::try_from(value).map_err(invalid_argument)
}
