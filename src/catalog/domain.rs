// Domain models for Catalog context (placeholder for richer invariants).
#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub vendor_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub id: String,
    pub product_id: String,
    pub sku: String,
    pub status: String,
}
