use anyhow::Result;
use clap::{Parser, Subcommand};
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder, postgres::PgPoolOptions};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "rs-ecommerce", version, about = "rs-ecommerce operational CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Search {
        #[command(subcommand)]
        command: SearchCommands,
    },
}

#[derive(Subcommand, Debug)]
enum SearchCommands {
    Reindex(ReindexArgs),
}

#[derive(Parser, Debug)]
struct ReindexArgs {
    #[arg(long, env = "DATABASE_URL")]
    db_url: String,
    #[arg(long, env = "MEILI_URL")]
    meili_url: String,
    #[arg(long, env = "MEILI_MASTER_KEY")]
    meili_key: Option<String>,
    #[arg(long, env = "MEILI_INDEX", default_value = "products")]
    index_name: String,
    #[arg(long, env = "REINDEX_BATCH_SIZE", default_value_t = 500)]
    batch_size: usize,
    #[arg(long, env = "REINDEX_DRY_RUN", default_value_t = false)]
    dry_run: bool,
    #[arg(long, env = "REINDEX_COUNT_ONLY", default_value_t = false)]
    count_only: bool,
    #[arg(long, env = "REINDEX_TENANT_ID")]
    tenant_id: Option<String>,
    #[arg(long, env = "REINDEX_STORE_ID")]
    store_id: Option<String>,
    #[arg(long, env = "REINDEX_VENDOR_ID")]
    vendor_id: Option<String>,
    #[arg(long, env = "REINDEX_STATUS")]
    status: Option<String>,
    #[arg(long, env = "REINDEX_PRODUCT_ID")]
    product_id: Option<String>,
}

#[derive(Debug, Default)]
struct ReindexFilters {
    tenant_id: Option<String>,
    store_id: Option<String>,
    vendor_id: Option<String>,
    status: Option<String>,
    product_id: Option<String>,
}

impl ReindexFilters {
    fn is_empty(&self) -> bool {
        self.tenant_id.is_none()
            && self.store_id.is_none()
            && self.vendor_id.is_none()
            && self.status.is_none()
            && self.product_id.is_none()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchProduct {
    id: String,
    tenant_id: String,
    #[serde(default)]
    store_id: String,
    #[serde(default)]
    vendor_id: String,
    title: String,
    description: String,
    status: String,
    #[serde(default)]
    primary_category_id: String,
    #[serde(default)]
    category_ids: Vec<String>,
    #[serde(default)]
    sku_codes: Vec<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct SearchRow {
    id: String,
    tenant_id: String,
    store_id: String,
    vendor_id: Option<String>,
    title: String,
    description: String,
    status: String,
    primary_category_id: String,
    category_ids: Vec<String>,
    sku_codes: Vec<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct CountRow {
    count: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    rs_common::telemetry::init_tracing("rs-ecommerce-cli");

    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            command: SearchCommands::Reindex(args),
        } => run_reindex(args).await,
    }
}

async fn run_reindex(args: ReindexArgs) -> Result<()> {
    let filters = ReindexFilters {
        tenant_id: args.tenant_id.filter(|v| !v.is_empty()),
        store_id: args.store_id.filter(|v| !v.is_empty()),
        vendor_id: args.vendor_id.filter(|v| !v.is_empty()),
        status: args.status.filter(|v| !v.is_empty()),
        product_id: args.product_id.filter(|v| !v.is_empty()),
    };

    let batch_size = args.batch_size as i64;
    let db = PgPoolOptions::new().max_connections(5).connect(&args.db_url).await?;

    tracing::info!(
        index = %args.index_name,
        batch_size,
        dry_run = args.dry_run,
        count_only = args.count_only,
        filters = ?filters,
        "search reindex started"
    );

    if args.count_only {
        let mut builder = QueryBuilder::<Postgres>::new("SELECT COUNT(*) as count FROM products p");
        if !filters.is_empty() {
            builder.push(" WHERE ");
            apply_filters(&mut builder, &filters);
        }
        let row = builder.build_query_as::<CountRow>().fetch_one(&db).await?;
        tracing::info!(count = row.count, filters = ?filters, "search reindex count-only");
        return Ok(());
    }

    if args.dry_run {
        tracing::info!("search reindex running in dry-run mode");
    }

    let client = Client::new(&args.meili_url, args.meili_key.map(|v| v.to_string()));
    let index = client.index(&args.index_name);
    index
        .set_filterable_attributes(&[
            "tenant_id",
            "store_id",
            "vendor_id",
            "status",
            "primary_category_id",
            "category_ids",
        ])
        .await?;

    let mut offset = 0i64;
    let mut total = 0usize;
    let started_at = Instant::now();
    loop {
        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT p.id::text as id,
                   p.tenant_id::text as tenant_id,
                   p.store_id::text as store_id,
                   p.vendor_id::text as vendor_id,
                   p.title,
                   p.description,
                   p.status,
                   COALESCE((
                       SELECT category_id::text
                       FROM product_category_links
                       WHERE product_id = p.id AND is_primary
                       LIMIT 1
                   ), '') as primary_category_id,
                   COALESCE((
                       SELECT array_agg(category_id::text ORDER BY position)
                       FROM product_category_links
                       WHERE product_id = p.id
                   ), ARRAY[]::text[]) as category_ids,
                   COALESCE((
                       SELECT array_agg(sku ORDER BY created_at)
                       FROM product_skus
                       WHERE product_id = p.id
                   ), ARRAY[]::text[]) as sku_codes
            FROM products p
            "#,
        );
        if !filters.is_empty() {
            builder.push(" WHERE ");
            apply_filters(&mut builder, &filters);
        }
        builder.push(" ORDER BY p.created_at ASC LIMIT ");
        builder.push_bind(batch_size);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        let rows = builder.build_query_as::<SearchRow>().fetch_all(&db).await?;

        if rows.is_empty() {
            break;
        }

        let rows_len = rows.len();
        let mut docs = Vec::with_capacity(rows_len);
        for row in rows {
            docs.push(SearchProduct {
                id: row.id,
                tenant_id: row.tenant_id,
                store_id: row.store_id,
                vendor_id: row.vendor_id.unwrap_or_default(),
                title: row.title,
                description: row.description,
                status: row.status,
                primary_category_id: row.primary_category_id,
                category_ids: row.category_ids,
                sku_codes: row.sku_codes,
            });
        }

        if !args.dry_run {
            index.add_or_replace(&docs, Some("id")).await?;
        }
        total += rows_len;
        offset += batch_size;
        tracing::info!(
            processed = total,
            batch_size = rows_len,
            offset,
            dry_run = args.dry_run,
            "search reindex batch completed"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    tracing::info!(
        total,
        dry_run = args.dry_run,
        elapsed_ms = started_at.elapsed().as_millis(),
        "search reindex completed"
    );
    Ok(())
}

fn apply_filters<'a>(builder: &mut QueryBuilder<'a, Postgres>, filters: &'a ReindexFilters) {
    let mut separated = builder.separated(" AND ");
    if let Some(tenant_id) = &filters.tenant_id {
        separated.push("p.tenant_id::text = ");
        separated.push_bind(tenant_id);
    }
    if let Some(store_id) = &filters.store_id {
        separated.push("p.store_id::text = ");
        separated.push_bind(store_id);
    }
    if let Some(vendor_id) = &filters.vendor_id {
        separated.push("p.vendor_id::text = ");
        separated.push_bind(vendor_id);
    }
    if let Some(status) = &filters.status {
        separated.push("p.status = ");
        separated.push_bind(status);
    }
    if let Some(product_id) = &filters.product_id {
        separated.push("p.id::text = ");
        separated.push_bind(product_id);
    }
}
