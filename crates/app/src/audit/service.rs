use axum::{Json, http::StatusCode};
use sqlx::{QueryBuilder, Row};

use crate::{
    AppState,
    infrastructure::db,
    pb::pb,
    rpc::json::ConnectError,
    shared::{
        ids::parse_uuid,
        time::{chrono_to_timestamp, timestamp_to_chrono},
    },
};

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub async fn list_audit_logs(
    state: &AppState,
    store_id: String,
    req: pb::ListAuditLogsRequest,
) -> Result<(Vec<pb::AuditLog>, pb::PageResult), (StatusCode, Json<ConnectError>)> {
    let (limit, offset) = page_params(req.page.clone());

    let mut qb = QueryBuilder::new(
        r#"
        SELECT id::text as id,
            store_id,
            actor_id,
            actor_type,
            action,
            target_type,
            target_id,
            request_id,
            ip_address,
            user_agent,
            before_json,
            after_json,
            metadata_json,
            created_at
        FROM audit_logs
        WHERE store_id = 
        "#,
    );
    qb.push_bind(parse_uuid(&store_id, "store_id")?);

    if !req.actor_id.is_empty() {
        qb.push(" AND actor_id = ").push_bind(req.actor_id);
    }
    if !req.actor_type.is_empty() {
        qb.push(" AND actor_type = ").push_bind(req.actor_type);
    }
    if !req.action.is_empty() {
        qb.push(" AND action = ").push_bind(req.action);
    }
    if !req.target_type.is_empty() {
        qb.push(" AND target_type = ").push_bind(req.target_type);
    }
    if !req.target_id.is_empty() {
        qb.push(" AND target_id = ").push_bind(req.target_id);
    }
    if let Some(from_time) = timestamp_to_chrono(req.from_time.clone()) {
        qb.push(" AND created_at >= ").push_bind(from_time);
    }
    if let Some(to_time) = timestamp_to_chrono(req.to_time.clone()) {
        qb.push(" AND created_at <= ").push_bind(to_time);
    }
    if !req.request_id.is_empty() {
        qb.push(" AND request_id = ").push_bind(req.request_id);
    }
    if !req.ip_address.is_empty() {
        qb.push(" AND ip_address = ").push_bind(req.ip_address);
    }
    if !req.user_agent.is_empty() {
        qb.push(" AND user_agent = ").push_bind(req.user_agent);
    }

    qb.push(" ORDER BY created_at DESC, id DESC");
    qb.push(" LIMIT ").push_bind(limit + 1);
    qb.push(" OFFSET ").push_bind(offset);

    let rows = qb.build().fetch_all(&state.db).await.map_err(db::error)?;

    let mut logs = Vec::new();
    for row in rows.into_iter() {
        logs.push(pb::AuditLog {
            id: row.get("id"),
            store_id: row.get::<uuid::Uuid, _>("store_id").to_string(),
            actor_id: row.get::<Option<String>, _>("actor_id").unwrap_or_default(),
            actor_type: row.get("actor_type"),
            action: row.get("action"),
            target_type: row.get::<Option<String>, _>("target_type").unwrap_or_default(),
            target_id: row.get::<Option<String>, _>("target_id").unwrap_or_default(),
            request_id: row.get::<Option<String>, _>("request_id").unwrap_or_default(),
            ip_address: row.get::<Option<String>, _>("ip_address").unwrap_or_default(),
            user_agent: row.get::<Option<String>, _>("user_agent").unwrap_or_default(),
            before_json: row
                .get::<Option<serde_json::Value>, _>("before_json")
                .map(|v| v.to_string())
                .unwrap_or_default(),
            after_json: row
                .get::<Option<serde_json::Value>, _>("after_json")
                .map(|v| v.to_string())
                .unwrap_or_default(),
            metadata_json: row
                .get::<Option<serde_json::Value>, _>("metadata_json")
                .map(|v| v.to_string())
                .unwrap_or_default(),
            created_at: chrono_to_timestamp(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")),
        });
    }

    let mut next_page_token = String::new();
    if logs.len() > limit as usize {
        logs.truncate(limit as usize);
        next_page_token = (offset + limit).to_string();
    }

    Ok((logs, pb::PageResult { next_page_token }))
}

fn page_params(page: Option<pb::PageInfo>) -> (i64, i64) {
    let page = page.unwrap_or(pb::PageInfo {
        page_size: DEFAULT_PAGE_SIZE as i32,
        page_token: String::new(),
    });
    let mut limit = page.page_size as i64;
    if limit <= 0 {
        limit = DEFAULT_PAGE_SIZE;
    } else if limit > MAX_PAGE_SIZE {
        limit = MAX_PAGE_SIZE;
    }
    let offset = page.page_token.parse::<i64>().unwrap_or(0).max(0);
    (limit, offset)
}
