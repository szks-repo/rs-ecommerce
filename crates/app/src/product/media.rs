use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream};
use axum::{Json, http::StatusCode};
use chrono::Utc;
use reqwest::{Url, header};
use sqlx::Row;
use std::{collections::HashMap, net::IpAddr, time::Duration};

use crate::{
    AppState,
    infrastructure::{db, storage},
    pb::pb,
    rpc::json::ConnectError,
    shared::{ids::parse_uuid, time::chrono_to_timestamp},
};

fn validate_store_asset_input(asset: &pb::MediaAsset) -> Result<(), (StatusCode, Json<ConnectError>)> {
    if asset.public_url.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "public_url is required".to_string(),
            }),
        ));
    }
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "upload".to_string();
    }
    let sanitized: String = trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    sanitized
}

fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            match octets {
                [10, ..] => true,
                [127, ..] => true,
                [169, 254, ..] => true,
                [172, b, ..] if (16..=31).contains(&b) => true,
                [192, 168, ..] => true,
                _ => false,
            }
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local() || v6.is_unspecified(),
    }
}

fn validate_external_url(input: &str) -> Result<Url, (StatusCode, Json<ConnectError>)> {
    let url = Url::parse(input).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "public_url must be a valid URL".to_string(),
            }),
        )
    })?;
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "public_url must be http or https".to_string(),
            }),
        ));
    }
    let host = url.host_str().unwrap_or_default().to_lowercase();
    if host == "localhost" || host.ends_with(".local") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "public_url host is not allowed".to_string(),
            }),
        ));
    }
    if let Some(ip) = url.host_str().and_then(|h| h.parse::<IpAddr>().ok())
        && is_private_ip(&ip)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "public_url host is not allowed".to_string(),
            }),
        ));
    }
    Ok(url)
}

fn extension_from_content_type(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        "image/avif" => Some("avif"),
        _ => None,
    }
}

async fn download_external_image(url: &Url) -> Result<(bytes::Bytes, String), (StatusCode, Json<ConnectError>)> {
    const MAX_BYTES: usize = 10 * 1024 * 1024;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::Internal,
                    message: "failed to create http client".to_string(),
                }),
            )
        })?;
    let resp = client
        .get(url.clone())
        .header(header::USER_AGENT, "rs-ecommerce-media-fetcher")
        .send()
        .await
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "failed to download image".to_string(),
                }),
            )
        })?;
    if !resp.status().is_success() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "failed to download image".to_string(),
            }),
        ));
    }
    if let Some(length) = resp.content_length()
        && length as usize > MAX_BYTES
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "image is too large".to_string(),
            }),
        ));
    }
    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if !content_type.starts_with("image/") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "public_url must point to an image".to_string(),
            }),
        ));
    }
    let bytes = resp.bytes().await.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "failed to download image".to_string(),
            }),
        )
    })?;
    if bytes.len() > MAX_BYTES {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "image is too large".to_string(),
            }),
        ));
    }
    Ok((bytes, content_type))
}

async fn import_external_asset(
    _state: &AppState,
    store_id: &str,
    tenant_id: &str,
    public_url: &str,
    object_key_hint: &str,
) -> Result<pb::MediaAsset, (StatusCode, Json<ConnectError>)> {
    let url = validate_external_url(public_url)?;
    let storage_config = storage::public_config();
    if !storage_config.is_configured() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::FailedPrecondition,
                message: "storage is not configured".to_string(),
            }),
        ));
    }
    let (bytes, content_type) = download_external_image(&url).await?;
    let extension = extension_from_content_type(&content_type).unwrap_or("bin");
    let filename = if object_key_hint.trim().is_empty() {
        format!("media/{}.{}", uuid::Uuid::new_v4(), extension)
    } else {
        object_key_hint.trim().to_string()
    };
    let object_key = storage::build_object_key(&storage_config.base_path, tenant_id, store_id, &filename);

    match storage_config.provider.as_str() {
        "s3" => {
            let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
            if !storage_config.region.is_empty() {
                loader = loader.region(aws_config::Region::new(storage_config.region.clone()));
            }
            let config = loader.load().await;
            let client = aws_sdk_s3::Client::new(&config);
            client
                .put_object()
                .bucket(&storage_config.bucket)
                .key(&object_key)
                .body(ByteStream::from(bytes.clone()))
                .content_type(content_type.clone())
                .send()
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ConnectError {
                            code: crate::rpc::json::ErrorCode::Internal,
                            message: "failed to upload image".to_string(),
                        }),
                    )
                })?;
        }
        "gcs" => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::FailedPrecondition,
                    message: "GCS import is not configured yet".to_string(),
                }),
            ));
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "unsupported storage provider".to_string(),
                }),
            ));
        }
    }

    let public_base = if storage_config.cdn_base_url.is_empty() {
        match storage_config.provider.as_str() {
            "s3" => format!("https://{}.s3.amazonaws.com", storage_config.bucket),
            _ => "".to_string(),
        }
    } else {
        storage_config.cdn_base_url.trim_end_matches('/').to_string()
    };
    let public_url = format!("{}/{}", public_base, object_key);
    Ok(pb::MediaAsset {
        id: "".to_string(),
        public_url,
        provider: storage_config.provider,
        bucket: storage_config.bucket,
        object_key,
        content_type,
        size_bytes: bytes.len() as i64,
        created_at: None,
        tags: Vec::new(),
    })
}

pub async fn create_media_upload_url(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    filename: String,
    content_type: String,
    size_bytes: i64,
) -> Result<pb::CreateMediaUploadUrlResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, store.clone(), tenant.clone()).await?;
    let storage_config = storage::public_config();
    if !storage_config.is_configured() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::FailedPrecondition,
                message: "storage provider and bucket are required".to_string(),
            }),
        ));
    }

    let file_name = sanitize_filename(&filename);
    let object_key = storage::build_object_key(
        &storage_config.base_path,
        &tenant_id,
        &store_id,
        &format!("{}-{}", uuid::Uuid::new_v4(), file_name),
    );

    match storage_config.provider.as_str() {
        "s3" => {
            let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
            if !storage_config.region.is_empty() {
                loader = loader.region(aws_config::Region::new(storage_config.region.clone()));
            }
            let config = loader.load().await;
            let client = aws_sdk_s3::Client::new(&config);
            let mut put_req = client.put_object().bucket(&storage_config.bucket).key(&object_key);
            if !content_type.is_empty() {
                put_req = put_req.content_type(content_type.clone());
            }
            if size_bytes > 0 {
                put_req = put_req.content_length(size_bytes);
            }
            let presign_config = PresigningConfig::expires_in(Duration::from_secs(900)).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::Internal,
                        message: "failed to create presign config".to_string(),
                    }),
                )
            })?;
            let presigned = put_req.presigned(presign_config).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::Internal,
                        message: "failed to presign upload url".to_string(),
                    }),
                )
            })?;
            let mut headers = HashMap::new();
            for (key, value) in presigned.headers() {
                headers.insert(key.to_string(), value.to_string());
            }
            let public_base = if storage_config.cdn_base_url.is_empty() {
                format!("https://{}.s3.amazonaws.com", storage_config.bucket)
            } else {
                storage_config.cdn_base_url.trim_end_matches('/').to_string()
            };
            let public_url = format!("{}/{}", public_base, object_key);
            Ok(pb::CreateMediaUploadUrlResponse {
                upload_url: presigned.uri().to_string(),
                headers,
                public_url,
                provider: storage_config.provider,
                bucket: storage_config.bucket,
                object_key,
            })
        }
        "gcs" => Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::FailedPrecondition,
                message: "GCS presign is not configured yet".to_string(),
            }),
        )),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "unsupported storage provider".to_string(),
            }),
        )),
    }
}
async fn resolve_store_context(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> Result<(String, String), (StatusCode, Json<ConnectError>)> {
    crate::identity::context::resolve_store_context(state, store, tenant).await
}

async fn ensure_sku_belongs_to_store(
    state: &AppState,
    sku_id: &str,
    store_id: &str,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let sku_uuid = parse_uuid(sku_id, "sku_id")?;
    let row = sqlx::query(
        r#"
        SELECT p.store_id::text as store_id
        FROM product_skus v
        JOIN products p ON p.id = v.product_id
        WHERE v.id = $1
        "#,
    )
    .bind(sku_uuid)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "sku_id not found".to_string(),
            }),
        ));
    };
    let found_store: String = row.get("store_id");
    if found_store != store_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::PermissionDenied,
                message: "sku_id does not belong to store".to_string(),
            }),
        ));
    }
    Ok(())
}

pub async fn list_media_assets(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    query: String,
) -> Result<Vec<pb::MediaAsset>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store.clone(), tenant.clone()).await?;
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let like_query = format!("%{}%", query.trim());
    let rows = if query.trim().is_empty() {
        sqlx::query(
            r#"
            SELECT id::text as id, public_url, provider, bucket, object_key, content_type, size_bytes, tags, created_at
            FROM store_media_assets
            WHERE store_id = $1
            ORDER BY created_at DESC
            LIMIT 200
            "#,
        )
        .bind(store_uuid)
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?
    } else {
        sqlx::query(
            r#"
            SELECT id::text as id, public_url, provider, bucket, object_key, content_type, size_bytes, tags, created_at
            FROM store_media_assets
            WHERE store_id = $1
              AND (public_url ILIKE $2 OR object_key ILIKE $2)
            ORDER BY created_at DESC
            LIMIT 200
            "#,
        )
        .bind(store_uuid)
        .bind(like_query)
        .fetch_all(&state.db)
        .await
        .map_err(db::error)?
    };

    let assets = rows
        .into_iter()
        .map(|row| pb::MediaAsset {
            id: row.get("id"),
            public_url: row.get("public_url"),
            provider: row.get("provider"),
            bucket: row.get("bucket"),
            object_key: row.get("object_key"),
            content_type: row.get::<Option<String>, _>("content_type").unwrap_or_default(),
            size_bytes: row.get::<Option<i64>, _>("size_bytes").unwrap_or_default(),
            tags: row.get::<Option<Vec<String>>, _>("tags").unwrap_or_default(),
            created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        })
        .collect();

    Ok(assets)
}

pub async fn create_media_asset(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    asset: pb::MediaAsset,
) -> Result<pb::MediaAsset, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, store, tenant).await?;
    let mut asset = asset;
    if asset.provider.is_empty() && !asset.public_url.is_empty() {
        let imported =
            import_external_asset(state, &store_id, &tenant_id, &asset.public_url, &asset.object_key).await?;
        asset = imported;
    }
    validate_store_asset_input(&asset)?;
    let tags = normalize_tags(asset.tags);
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let tenant_uuid = parse_uuid(&tenant_id, "tenant_id")?;
    let now = Utc::now();

    let row = sqlx::query(
        r#"
        INSERT INTO store_media_assets (
            tenant_id, store_id, provider, bucket, object_key, public_url, content_type, size_bytes, tags, created_at
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        RETURNING id::text as id, public_url, provider, bucket, object_key, content_type, size_bytes, tags, created_at
        "#,
    )
    .bind(tenant_uuid)
    .bind(store_uuid)
    .bind(asset.provider)
    .bind(asset.bucket)
    .bind(asset.object_key)
    .bind(asset.public_url)
    .bind(if asset.content_type.is_empty() {
        None
    } else {
        Some(asset.content_type)
    })
    .bind(if asset.size_bytes <= 0 {
        None
    } else {
        Some(asset.size_bytes)
    })
    .bind(tags)
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::MediaAsset {
        id: row.get("id"),
        public_url: row.get("public_url"),
        provider: row.get("provider"),
        bucket: row.get("bucket"),
        object_key: row.get("object_key"),
        content_type: row.get::<Option<String>, _>("content_type").unwrap_or_default(),
        size_bytes: row.get::<Option<i64>, _>("size_bytes").unwrap_or_default(),
        tags: row.get::<Option<Vec<String>>, _>("tags").unwrap_or_default(),
        created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
    })
}

pub async fn update_media_asset_tags(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    asset_id: String,
    tags: Vec<String>,
) -> Result<pb::MediaAsset, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let tenant_uuid = parse_uuid(&tenant_id, "tenant_id")?;
    let asset_uuid = parse_uuid(&asset_id, "asset_id")?;
    let tags = normalize_tags(tags);
    let row = sqlx::query(
        r#"
        UPDATE store_media_assets
           SET tags = $1
         WHERE id = $2 AND store_id = $3 AND tenant_id = $4
         RETURNING id::text as id, public_url, provider, bucket, object_key, content_type, size_bytes, tags, created_at
        "#,
    )
    .bind(tags)
    .bind(asset_uuid)
    .bind(store_uuid)
    .bind(tenant_uuid)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::NotFound,
                message: "asset not found".to_string(),
            }),
        ));
    };

    Ok(pb::MediaAsset {
        id: row.get("id"),
        public_url: row.get("public_url"),
        provider: row.get("provider"),
        bucket: row.get("bucket"),
        object_key: row.get("object_key"),
        content_type: row.get::<Option<String>, _>("content_type").unwrap_or_default(),
        size_bytes: row.get::<Option<i64>, _>("size_bytes").unwrap_or_default(),
        tags: row.get::<Option<Vec<String>>, _>("tags").unwrap_or_default(),
        created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
    })
}

pub async fn delete_media_asset(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    asset_id: String,
) -> Result<bool, (StatusCode, Json<ConnectError>)> {
    let (store_id, tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let tenant_uuid = parse_uuid(&tenant_id, "tenant_id")?;
    let asset_uuid = parse_uuid(&asset_id, "asset_id")?;

    let used_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sku_images WHERE asset_id = $1")
        .bind(asset_uuid)
        .fetch_one(&state.db)
        .await
        .map_err(db::error)?;
    if used_count > 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "asset is used by sku images".to_string(),
            }),
        ));
    }

    let rows = sqlx::query("DELETE FROM store_media_assets WHERE id = $1 AND store_id = $2 AND tenant_id = $3")
        .bind(asset_uuid)
        .bind(store_uuid)
        .bind(tenant_uuid)
        .execute(&state.db)
        .await
        .map_err(db::error)?;
    Ok(rows.rows_affected() > 0)
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(50)
        .collect()
}

pub async fn list_sku_images(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    sku_id: String,
) -> Result<Vec<pb::SkuImage>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store.clone(), tenant.clone()).await?;
    ensure_sku_belongs_to_store(state, &sku_id, &store_id).await?;
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let sku_uuid = parse_uuid(&sku_id, "sku_id")?;

    let rows = sqlx::query(
        r#"
        SELECT si.asset_id::text as asset_id, si.position,
               a.id::text as asset_ref_id, a.public_url, a.provider, a.bucket, a.object_key,
               a.content_type, a.size_bytes, a.created_at
        FROM sku_images si
        JOIN store_media_assets a ON a.id = si.asset_id
        WHERE si.store_id = $1 AND si.sku_id = $2
        ORDER BY si.position ASC
        "#,
    )
    .bind(store_uuid)
    .bind(sku_uuid)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    let images = rows
        .into_iter()
        .map(|row| {
            let asset = pb::MediaAsset {
                id: row.get("asset_ref_id"),
                public_url: row.get("public_url"),
                provider: row.get("provider"),
                bucket: row.get("bucket"),
                object_key: row.get("object_key"),
                content_type: row.get::<Option<String>, _>("content_type").unwrap_or_default(),
                size_bytes: row.get::<Option<i64>, _>("size_bytes").unwrap_or_default(),
                created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
            };
            pb::SkuImage {
                asset_id: row.get("asset_id"),
                public_url: asset.public_url.clone(),
                position: row.get::<i32, _>("position"),
                asset: Some(asset),
            }
        })
        .collect();

    Ok(images)
}

pub async fn set_sku_images(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    sku_id: String,
    images: Vec<pb::SkuImageInput>,
) -> Result<Vec<pb::SkuImage>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store.clone(), tenant.clone()).await?;
    ensure_sku_belongs_to_store(state, &sku_id, &store_id).await?;
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let sku_uuid = parse_uuid(&sku_id, "sku_id")?;

    let mut tx = state.db.begin().await.map_err(db::error)?;
    sqlx::query("DELETE FROM sku_images WHERE store_id = $1 AND sku_id = $2")
        .bind(store_uuid)
        .bind(sku_uuid)
        .execute(tx.as_mut())
        .await
        .map_err(db::error)?;

    for image in images.iter() {
        let asset_uuid = parse_uuid(&image.asset_id, "asset_id")?;
        let exists = sqlx::query("SELECT 1 FROM store_media_assets WHERE id = $1 AND store_id = $2")
            .bind(asset_uuid)
            .bind(store_uuid)
            .fetch_optional(tx.as_mut())
            .await
            .map_err(db::error)?;
        if exists.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::InvalidArgument,
                    message: "asset_id not found".to_string(),
                }),
            ));
        }
        sqlx::query(
            r#"
            INSERT INTO sku_images (store_id, sku_id, asset_id, position)
            VALUES ($1,$2,$3,$4)
            "#,
        )
        .bind(store_uuid)
        .bind(sku_uuid)
        .bind(asset_uuid)
        .bind(image.position)
        .execute(tx.as_mut())
        .await
        .map_err(db::error)?;
    }

    tx.commit().await.map_err(db::error)?;

    list_sku_images(state, store, tenant, sku_id).await
}
