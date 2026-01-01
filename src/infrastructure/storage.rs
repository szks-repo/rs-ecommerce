#[derive(Debug, Clone)]
pub struct StorageLocationConfig {
    pub provider: String,
    pub bucket: String,
    pub base_path: String,
    pub cdn_base_url: String,
    pub region: String,
}

impl StorageLocationConfig {
    pub fn is_configured(&self) -> bool {
        !self.provider.trim().is_empty() && !self.bucket.trim().is_empty()
    }
}

fn read_env(key: &str) -> String {
    std::env::var(key).unwrap_or_default().trim().to_string()
}

pub fn public_config() -> StorageLocationConfig {
    StorageLocationConfig {
        provider: read_env("STORAGE_PUBLIC_PROVIDER"),
        bucket: read_env("STORAGE_PUBLIC_BUCKET"),
        base_path: read_env("STORAGE_PUBLIC_BASE_PATH"),
        cdn_base_url: read_env("STORAGE_PUBLIC_CDN_BASE_URL"),
        region: read_env("STORAGE_PUBLIC_REGION"),
    }
}

pub fn private_config() -> StorageLocationConfig {
    StorageLocationConfig {
        provider: read_env("STORAGE_PRIVATE_PROVIDER"),
        bucket: read_env("STORAGE_PRIVATE_BUCKET"),
        base_path: read_env("STORAGE_PRIVATE_BASE_PATH"),
        cdn_base_url: read_env("STORAGE_PRIVATE_CDN_BASE_URL"),
        region: read_env("STORAGE_PRIVATE_REGION"),
    }
}

pub fn join_path(base: &str, suffix: &str) -> String {
    let base = base.trim().trim_matches('/');
    if base.is_empty() {
        return suffix.to_string();
    }
    format!("{}/{}", base, suffix)
}

pub fn store_prefix(base_path: &str, tenant_id: &str, store_id: &str) -> String {
    let scoped = format!("{}/{}", tenant_id.trim(), store_id.trim());
    join_path(base_path, &scoped)
}

pub fn build_object_key(base_path: &str, tenant_id: &str, store_id: &str, name: &str) -> String {
    let prefix = store_prefix(base_path, tenant_id, store_id);
    join_path(&prefix, name.trim())
}
