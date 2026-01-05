use axum::{Json, http::StatusCode};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, postgres::PgRow};

use crate::{infrastructure::db, rpc::json::ConnectError};

#[derive(Debug, Clone)]
pub struct MetafieldDefinitionRecord {
    pub id: String,
    pub owner_type: String,
    pub namespace: String,
    pub key: String,
    pub name: String,
    pub description: String,
    pub value_type: String,
    pub is_list: bool,
    pub validations_json: String,
    pub visibility_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MetafieldValueRecord {
    pub id: String,
    pub definition_id: String,
    pub owner_id: String,
    pub value_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub definition: MetafieldDefinitionRecord,
}

#[derive(Debug, Clone)]
pub struct MetafieldDefinitionInput {
    pub namespace: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub value_type: String,
    pub is_list: bool,
    pub validations_json: String,
    pub visibility_json: String,
}

fn definition_from_row(row: &PgRow) -> MetafieldDefinitionRecord {
    MetafieldDefinitionRecord {
        id: row.get("id"),
        owner_type: row.get("owner_type"),
        namespace: row.get("namespace"),
        key: row.get("key"),
        name: row.get("name"),
        description: row.get::<Option<String>, _>("description").unwrap_or_default(),
        value_type: row.get("value_type"),
        is_list: row.get("is_list"),
        validations_json: row
            .get::<Option<String>, _>("validations_json")
            .unwrap_or_else(|| "{}".to_string()),
        visibility_json: row
            .get::<Option<String>, _>("visibility_json")
            .unwrap_or_else(|| "{}".to_string()),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    }
}

pub fn normalize_optional_json(value: String) -> Result<String, (StatusCode, Json<ConnectError>)> {
    if value.trim().is_empty() {
        return Ok("{}".to_string());
    }
    if serde_json::from_str::<serde_json::Value>(&value).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "invalid json provided".to_string(),
            }),
        ));
    }
    Ok(value)
}

pub async fn list_definitions(
    pool: &PgPool,
    owner_type: &str,
) -> Result<Vec<MetafieldDefinitionRecord>, (StatusCode, Json<ConnectError>)> {
    let rows = sqlx::query(
        r#"
        SELECT id::text as id,
               owner_type,
               namespace,
               key,
               name,
               description,
               value_type,
               is_list,
               validations_json::text as validations_json,
               visibility_json::text as visibility_json,
               created_at,
               updated_at
        FROM metafield_definitions
        WHERE owner_type = $1
        ORDER BY namespace ASC, key ASC
        "#,
    )
    .bind(owner_type)
    .fetch_all(pool)
    .await
    .map_err(db::error)?;

    Ok(rows.into_iter().map(|row| definition_from_row(&row)).collect())
}

pub async fn create_definition(
    pool: &PgPool,
    owner_type: &str,
    input: MetafieldDefinitionInput,
) -> Result<MetafieldDefinitionRecord, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        INSERT INTO metafield_definitions
            (owner_type, namespace, key, name, description, value_type, is_list, validations_json, visibility_json)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8::jsonb,$9::jsonb)
        RETURNING id::text as id,
                  owner_type,
                  namespace,
                  key,
                  name,
                  description,
                  value_type,
                  is_list,
                  validations_json::text as validations_json,
                  visibility_json::text as visibility_json,
                  created_at,
                  updated_at
        "#,
    )
    .bind(owner_type)
    .bind(input.namespace)
    .bind(input.key)
    .bind(input.name)
    .bind(input.description)
    .bind(input.value_type)
    .bind(input.is_list)
    .bind(input.validations_json)
    .bind(input.visibility_json)
    .fetch_one(pool)
    .await
    .map_err(db::error)?;

    Ok(definition_from_row(&row))
}

pub async fn update_definition(
    pool: &PgPool,
    owner_type: &str,
    definition_id: &uuid::Uuid,
    input: MetafieldDefinitionInput,
) -> Result<Option<MetafieldDefinitionRecord>, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        UPDATE metafield_definitions
        SET namespace = $1,
            key = $2,
            name = $3,
            description = $4,
            value_type = $5,
            is_list = $6,
            validations_json = $7::jsonb,
            visibility_json = $8::jsonb,
            updated_at = now()
        WHERE id = $9 AND owner_type = $10
        RETURNING id::text as id,
                  owner_type,
                  namespace,
                  key,
                  name,
                  description,
                  value_type,
                  is_list,
                  validations_json::text as validations_json,
                  visibility_json::text as visibility_json,
                  created_at,
                  updated_at
        "#,
    )
    .bind(input.namespace)
    .bind(input.key)
    .bind(input.name)
    .bind(input.description)
    .bind(input.value_type)
    .bind(input.is_list)
    .bind(input.validations_json)
    .bind(input.visibility_json)
    .bind(definition_id)
    .bind(owner_type)
    .fetch_optional(pool)
    .await
    .map_err(db::error)?;

    Ok(row.map(|row| definition_from_row(&row)))
}

pub async fn fetch_definition(
    pool: &PgPool,
    owner_type: &str,
    definition_id: &uuid::Uuid,
) -> Result<Option<MetafieldDefinitionRecord>, (StatusCode, Json<ConnectError>)> {
    let row = sqlx::query(
        r#"
        SELECT id::text as id,
               owner_type,
               namespace,
               key,
               name,
               description,
               value_type,
               is_list,
               validations_json::text as validations_json,
               visibility_json::text as visibility_json,
               created_at,
               updated_at
        FROM metafield_definitions
        WHERE id = $1 AND owner_type = $2
        "#,
    )
    .bind(definition_id)
    .bind(owner_type)
    .fetch_optional(pool)
    .await
    .map_err(db::error)?;

    Ok(row.map(|row| definition_from_row(&row)))
}

pub async fn list_values(
    pool: &PgPool,
    owner_type: &str,
    owner_id: &uuid::Uuid,
) -> Result<Vec<MetafieldValueRecord>, (StatusCode, Json<ConnectError>)> {
    let rows = sqlx::query(
        r#"
        SELECT v.id::text as id,
               v.definition_id::text as definition_id,
               v.owner_id::text as owner_id,
               v.value_json::text as value_json,
               v.created_at,
               v.updated_at,
               d.id::text as def_id,
               d.owner_type,
               d.namespace,
               d.key,
               d.name,
               d.description,
               d.value_type,
               d.is_list,
               d.validations_json::text as validations_json,
               d.visibility_json::text as visibility_json,
               d.created_at as def_created_at,
               d.updated_at as def_updated_at
        FROM metafield_values v
        JOIN metafield_definitions d ON d.id = v.definition_id
        WHERE d.owner_type = $1
          AND v.owner_id = $2
        ORDER BY d.namespace ASC, d.key ASC
        "#,
    )
    .bind(owner_type)
    .bind(owner_id)
    .fetch_all(pool)
    .await
    .map_err(db::error)?;

    let values = rows
        .into_iter()
        .map(|row| {
            let definition = MetafieldDefinitionRecord {
                id: row.get("def_id"),
                owner_type: row.get("owner_type"),
                namespace: row.get("namespace"),
                key: row.get("key"),
                name: row.get("name"),
                description: row.get::<Option<String>, _>("description").unwrap_or_default(),
                value_type: row.get("value_type"),
                is_list: row.get("is_list"),
                validations_json: row
                    .get::<Option<String>, _>("validations_json")
                    .unwrap_or_else(|| "{}".to_string()),
                visibility_json: row
                    .get::<Option<String>, _>("visibility_json")
                    .unwrap_or_else(|| "{}".to_string()),
                created_at: row.get::<DateTime<Utc>, _>("def_created_at"),
                updated_at: row.get::<DateTime<Utc>, _>("def_updated_at"),
            };
            MetafieldValueRecord {
                id: row.get("id"),
                definition_id: row.get("definition_id"),
                owner_id: row.get("owner_id"),
                value_json: row.get::<Option<String>, _>("value_json").unwrap_or_default(),
                created_at: row.get::<DateTime<Utc>, _>("created_at"),
                updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                definition,
            }
        })
        .collect();

    Ok(values)
}

pub async fn upsert_value(
    pool: &PgPool,
    definition_id: &uuid::Uuid,
    owner_id: &uuid::Uuid,
    value_json: &str,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    sqlx::query(
        r#"
        INSERT INTO metafield_values (definition_id, owner_id, value_json)
        VALUES ($1, $2, $3::jsonb)
        ON CONFLICT (definition_id, owner_id)
        DO UPDATE SET value_json = EXCLUDED.value_json, updated_at = now()
        "#,
    )
    .bind(definition_id)
    .bind(owner_id)
    .bind(value_json)
    .execute(pool)
    .await
    .map_err(db::error)?;

    Ok(())
}
