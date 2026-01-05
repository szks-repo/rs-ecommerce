use aws_sdk_s3::presigning::PresigningConfig;
use axum::{Json, http::StatusCode};
use chrono::{Duration as ChronoDuration, Utc};
use sqlx::Row;
use std::{collections::HashMap, time::Duration};

use crate::{
    AppState,
    infrastructure::{db, storage},
    pb::pb,
    rpc::json::ConnectError,
    shared::{ids::parse_uuid, status::FulfillmentType, time::chrono_to_timestamp},
};

async fn resolve_store_context(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
) -> Result<(String, String), (StatusCode, Json<ConnectError>)> {
    crate::identity::context::resolve_store_context(state, store, tenant).await
}

async fn ensure_sku_is_digital(
    state: &AppState,
    sku_id: &str,
    store_id: &str,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let sku_uuid = parse_uuid(sku_id, "sku_id")?;
    let store_uuid = parse_uuid(store_id, "store_id")?;
    let row = sqlx::query(
        r#"
        SELECT v.fulfillment_type
        FROM product_skus v
        JOIN products p ON p.id = v.product_id
        WHERE v.id = $1 AND p.store_id = $2
        "#,
    )
    .bind(sku_uuid)
    .bind(store_uuid)
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
    let fulfillment_type: String = row.get("fulfillment_type");
    let parsed = FulfillmentType::parse(&fulfillment_type)?;
    if parsed != FulfillmentType::Digital {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::FailedPrecondition,
                message: "sku is not digital fulfillment".to_string(),
            }),
        ));
    }
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "download".to_string();
    }
    trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub async fn list_digital_assets(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    sku_id: String,
) -> Result<Vec<pb::DigitalAsset>, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    ensure_sku_is_digital(state, &sku_id, &store_id).await?;
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let sku_uuid = parse_uuid(&sku_id, "sku_id")?;
    let rows = sqlx::query(
        r#"
        SELECT id::text as id, sku_id::text as sku_id, provider, bucket, object_key,
               content_type, size_bytes, created_at
        FROM store_digital_assets
        WHERE store_id = $1 AND sku_id = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(store_uuid)
    .bind(sku_uuid)
    .fetch_all(&state.db)
    .await
    .map_err(db::error)?;

    let assets = rows
        .into_iter()
        .map(|row| pb::DigitalAsset {
            id: row.get("id"),
            sku_id: row.get("sku_id"),
            provider: row.get("provider"),
            bucket: row.get("bucket"),
            object_key: row.get("object_key"),
            content_type: row.get::<Option<String>, _>("content_type").unwrap_or_default(),
            size_bytes: row.get::<Option<i64>, _>("size_bytes").unwrap_or_default(),
            created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
        })
        .collect();

    Ok(assets)
}

pub async fn create_digital_upload_url(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    sku_id: String,
    filename: String,
    content_type: String,
    _size_bytes: i64,
) -> Result<pb::CreateDigitalUploadUrlResponse, (StatusCode, Json<ConnectError>)> {
    if filename.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "filename is required".to_string(),
            }),
        ));
    }
    let (store_id, tenant_id) = resolve_store_context(state, store, tenant).await?;
    ensure_sku_is_digital(state, &sku_id, &store_id).await?;
    let storage_config = storage::private_config();
    if !storage_config.is_configured() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::FailedPrecondition,
                message: "private storage is not configured".to_string(),
            }),
        ));
    }

    let safe_name = sanitize_filename(&filename);
    let object_key = storage::build_object_key(
        &storage_config.base_path,
        &tenant_id,
        &store_id,
        &format!("{}-{}", uuid::Uuid::new_v4(), safe_name),
    );

    match storage_config.provider.as_str() {
        "s3" => {
            let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
            if !storage_config.region.is_empty() {
                loader = loader.region(aws_config::Region::new(storage_config.region.clone()));
            }
            let config = loader.load().await;
            let client = aws_sdk_s3::Client::new(&config);
            let put_req = client
                .put_object()
                .bucket(&storage_config.bucket)
                .key(&object_key)
                .content_type(content_type.clone());
            let presign_config = PresigningConfig::expires_in(Duration::from_secs(600)).map_err(|_| {
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
            Ok(pb::CreateDigitalUploadUrlResponse {
                upload_url: presigned.uri().to_string(),
                headers,
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

pub async fn create_digital_asset(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    sku_id: String,
    mut asset: pb::DigitalAsset,
) -> Result<pb::DigitalAsset, (StatusCode, Json<ConnectError>)> {
    if asset.object_key.is_empty() || asset.bucket.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "asset.object_key and bucket are required".to_string(),
            }),
        ));
    }
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    ensure_sku_is_digital(state, &sku_id, &store_id).await?;
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let sku_uuid = parse_uuid(&sku_id, "sku_id")?;
    let now = Utc::now();
    asset.sku_id = sku_id.clone();

    let row = sqlx::query(
        r#"
        INSERT INTO store_digital_assets (
            store_id, sku_id, provider, bucket, object_key, content_type, size_bytes, created_at
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        RETURNING id::text as id, sku_id::text as sku_id, provider, bucket, object_key, content_type, size_bytes, created_at
        "#,
    )
    .bind(store_uuid)
    .bind(sku_uuid)
    .bind(&asset.provider)
    .bind(&asset.bucket)
    .bind(&asset.object_key)
    .bind(if asset.content_type.is_empty() {
        None
    } else {
        Some(asset.content_type)
    })
    .bind(if asset.size_bytes <= 0 { None } else { Some(asset.size_bytes) })
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map_err(db::error)?;

    Ok(pb::DigitalAsset {
        id: row.get("id"),
        sku_id: row.get("sku_id"),
        provider: row.get("provider"),
        bucket: row.get("bucket"),
        object_key: row.get("object_key"),
        content_type: row.get::<Option<String>, _>("content_type").unwrap_or_default(),
        size_bytes: row.get::<Option<i64>, _>("size_bytes").unwrap_or_default(),
        created_at: chrono_to_timestamp(Some(row.get::<chrono::DateTime<Utc>, _>("created_at"))),
    })
}

pub async fn create_digital_download_url(
    state: &AppState,
    store: Option<pb::StoreContext>,
    tenant: Option<pb::TenantContext>,
    asset_id: String,
) -> Result<pb::CreateDigitalDownloadUrlResponse, (StatusCode, Json<ConnectError>)> {
    let (store_id, _tenant_id) = resolve_store_context(state, store, tenant).await?;
    let store_uuid = parse_uuid(&store_id, "store_id")?;
    let asset_uuid = parse_uuid(&asset_id, "asset_id")?;
    let row = sqlx::query(
        r#"
        SELECT provider, bucket, object_key
        FROM store_digital_assets
        WHERE id = $1 AND store_id = $2
        "#,
    )
    .bind(asset_uuid)
    .bind(store_uuid)
    .fetch_optional(&state.db)
    .await
    .map_err(db::error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "asset_id not found".to_string(),
            }),
        ));
    };
    let provider: String = row.get("provider");
    let bucket: String = row.get("bucket");
    let object_key: String = row.get("object_key");

    let storage_config = storage::private_config();
    if !storage_config.is_configured() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::FailedPrecondition,
                message: "private storage is not configured".to_string(),
            }),
        ));
    }
    match provider.as_str() {
        "s3" => {
            let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
            if !storage_config.region.is_empty() {
                loader = loader.region(aws_config::Region::new(storage_config.region.clone()));
            }
            let config = loader.load().await;
            let client = aws_sdk_s3::Client::new(&config);
            let get_req = client.get_object().bucket(&bucket).key(&object_key);
            let presign_config = PresigningConfig::expires_in(Duration::from_secs(3600)).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::Internal,
                        message: "failed to create presign config".to_string(),
                    }),
                )
            })?;
            let presigned = get_req.presigned(presign_config).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::Internal,
                        message: "failed to presign download url".to_string(),
                    }),
                )
            })?;
            let expires_at = Utc::now() + ChronoDuration::hours(1);
            Ok(pb::CreateDigitalDownloadUrlResponse {
                download_url: presigned.uri().to_string(),
                expires_at: chrono_to_timestamp(Some(expires_at)),
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
