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
                        code: "internal",
                        message: format!("search error: {}", err),
                    }),
                )
            })?;
        Ok(results.hits.into_iter().map(|hit| hit.result).collect())
    }

    pub async fn upsert_products(
        &self,
        products: &[SearchProduct],
    ) -> Result<(), (StatusCode, Json<ConnectError>)> {
        let index = self.client.index(self.index_name.as_str());
        index
            .add_or_replace(products, Some("id"))
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ConnectError {
                        code: "internal",
                        message: format!("search upsert error: {}", err),
                    }),
                )
            })?;
        Ok(())
    }

    pub async fn ensure_settings(&self) -> Result<(), (StatusCode, Json<ConnectError>)> {
        let index = self.client.index(self.index_name.as_str());
        index
            .set_filterable_attributes(&["tenant_id", "vendor_id", "status"])
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ConnectError {
                        code: "internal",
                        message: format!("search settings error: {}", err),
                    }),
                )
            })?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SearchProduct {
    pub id: String,
    pub tenant_id: String,
    #[serde(default)]
    pub vendor_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
}
