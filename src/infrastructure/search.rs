use axum::{Json, http::StatusCode};

use crate::rpc::json::ConnectError;

#[derive(Clone)]
pub struct SearchClient {
    client: meilisearch_sdk::client::Client,
    index_name: String,
}

impl SearchClient {
    pub fn new(base_url: &str, api_key: Option<&str>, index_name: &str) -> Self {
        let client = meilisearch_sdk::client::Client::new(base_url, api_key.map(|v| v.to_string()));
        Self {
            client,
            index_name: index_name.to_string(),
        }
    }

    pub async fn search_products(
        &self,
        query: &str,
        limit: usize,
        tenant_id: &str,
    ) -> Result<Vec<SearchProduct>, (StatusCode, Json<ConnectError>)> {
        let index = self.client.index(self.index_name.as_str());
        let filter = format!("tenant_id = \"{}\"", tenant_id);
        let results = index
            .search()
            .with_query(query)
            .with_limit(limit)
            .with_filter(&filter)
            .execute::<SearchProduct>()
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::Internal,
                        message: format!("search error: {}", err),
                    }),
                )
            })?;
        Ok(results.hits.into_iter().map(|hit| hit.result).collect())
    }

    pub async fn upsert_products(&self, products: &[SearchProduct]) -> Result<(), (StatusCode, Json<ConnectError>)> {
        let index = self.client.index(self.index_name.as_str());
        index.add_or_replace(products, Some("id")).await.map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::Internal,
                    message: format!("search upsert error: {}", err),
                }),
            )
        })?;
        Ok(())
    }

    pub async fn ensure_settings(&self) -> Result<(), (StatusCode, Json<ConnectError>)> {
        let index = self.client.index(self.index_name.as_str());
        index
            .set_filterable_attributes(&[
                "tenant_id",
                "store_id",
                "vendor_id",
                "status",
                "primary_category_id",
                "category_ids",
            ])
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ConnectError {
                        code: crate::rpc::json::ErrorCode::Internal,
                        message: format!("search settings error: {}", err),
                    }),
                )
            })?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct OpenSearchClient {
    base_url: String,
    index_name: String,
}

impl OpenSearchClient {
    pub fn new(base_url: &str, index_name: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            index_name: index_name.to_string(),
        }
    }
}

#[derive(Clone)]
pub enum SearchBackend {
    Meilisearch(SearchClient),
    OpenSearch(OpenSearchClient),
    None,
}

#[derive(Clone)]
pub struct SearchService {
    backend: SearchBackend,
}

impl SearchService {
    pub fn meilisearch(base_url: &str, api_key: Option<&str>, index_name: &str) -> Self {
        Self {
            backend: SearchBackend::Meilisearch(SearchClient::new(base_url, api_key, index_name)),
        }
    }

    pub fn none() -> Self {
        Self {
            backend: SearchBackend::None,
        }
    }

    pub fn opensearch(base_url: &str, index_name: &str) -> Self {
        tracing::warn!(
            base_url,
            index_name,
            "opensearch backend selected but not implemented; falling back to noop"
        );
        Self {
            backend: SearchBackend::OpenSearch(OpenSearchClient::new(base_url, index_name)),
        }
    }

    pub async fn search_products(
        &self,
        query: &str,
        limit: usize,
        tenant_id: &str,
    ) -> Result<Vec<SearchProduct>, (StatusCode, Json<ConnectError>)> {
        match &self.backend {
            SearchBackend::Meilisearch(client) => client.search_products(query, limit, tenant_id).await,
            SearchBackend::OpenSearch(_) => Ok(Vec::new()),
            SearchBackend::None => Ok(Vec::new()),
        }
    }

    pub async fn upsert_products(&self, products: &[SearchProduct]) -> Result<(), (StatusCode, Json<ConnectError>)> {
        match &self.backend {
            SearchBackend::Meilisearch(client) => client.upsert_products(products).await,
            SearchBackend::OpenSearch(_) => Ok(()),
            SearchBackend::None => Ok(()),
        }
    }

    pub async fn ensure_settings(&self) -> Result<(), (StatusCode, Json<ConnectError>)> {
        match &self.backend {
            SearchBackend::Meilisearch(client) => client.ensure_settings().await,
            SearchBackend::OpenSearch(_) => Ok(()),
            SearchBackend::None => Ok(()),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SearchProduct {
    pub id: String,
    pub tenant_id: String,
    #[serde(default)]
    pub store_id: String,
    #[serde(default)]
    pub vendor_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub primary_category_id: String,
    #[serde(default)]
    pub category_ids: Vec<String>,
    #[serde(default)]
    pub sku_codes: Vec<String>,
}
