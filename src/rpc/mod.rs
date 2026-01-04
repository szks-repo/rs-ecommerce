use axum::{
    Router,
    extract::State,
    http::{HeaderValue, Method, Request},
    middleware,
    routing::post,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info_span;

use crate::AppState;

pub(crate) mod actor;
mod auction;
mod audit;
mod backoffice;
mod customer;
mod identity;
pub mod json;
mod permissions;
pub mod request_context;
mod session;
mod setup;
mod store_settings;
mod storefront;

pub fn router(state: AppState) -> Router<()> {
    let cors = CorsLayer::new()
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::HeaderName::from_static("x-actor-id"),
            axum::http::header::HeaderName::from_static("x-actor-type"),
            axum::http::header::HeaderName::from_static("x-request-id"),
            axum::http::header::HeaderName::from_static("x-refresh-attempt"),
            axum::http::header::HeaderName::from_static("connect-protocol-version"),
            axum::http::header::HeaderName::from_static("connect-timeout-ms"),
        ])
        .allow_credentials(true)
        .allow_origin([
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("http://127.0.0.1:3000"),
        ]);

    Router::new()
        .route(
            "/rpc/ecommerce.v1.StorefrontService/ListProducts",
            post(storefront::list_products),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/GetProduct",
            post(storefront::get_product),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/SearchProducts",
            post(storefront::search_products),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/CreateCart",
            post(storefront::create_cart),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/AddCartItem",
            post(storefront::add_cart_item),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/UpdateCartItem",
            post(storefront::update_cart_item),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/RemoveCartItem",
            post(storefront::remove_cart_item),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/GetCart",
            post(storefront::get_cart),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/Checkout",
            post(storefront::checkout),
        )
        .route(
            "/rpc/ecommerce.v1.StorefrontService/GetOrder",
            post(storefront::get_order),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ListProducts",
            post(backoffice::list_products).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateProduct",
            post(backoffice::create_product).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/UpdateProduct",
            post(backoffice::update_product).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ListCategories",
            post(backoffice::list_categories).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateCategory",
            post(backoffice::create_category).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/UpdateCategory",
            post(backoffice::update_category).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/DeleteCategory",
            post(backoffice::delete_category).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ReorderCategories",
            post(backoffice::reorder_categories).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ListVariants",
            post(backoffice::list_variants).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ListSkus",
            post(backoffice::list_skus).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/ListAuctions",
            post(auction::list_auctions).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/GetAuction",
            post(auction::get_auction).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/ListBids",
            post(auction::list_bids).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/ListAutoBids",
            post(auction::list_auto_bids).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/CreateAuction",
            post(auction::create_auction).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/UpdateAuction",
            post(auction::update_auction).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/PlaceBid",
            post(auction::place_bid).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/SetAutoBid",
            post(auction::set_auto_bid).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/CloseAuction",
            post(auction::close_auction).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuctionService/ApproveAuction",
            post(auction::approve_auction).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuctionWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateVariant",
            post(backoffice::create_variant).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/UpdateVariant",
            post(backoffice::update_variant).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ListMediaAssets",
            post(backoffice::list_media_assets).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateMediaAsset",
            post(backoffice::create_media_asset).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateMediaUploadUrl",
            post(backoffice::create_media_upload_url).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CatalogWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ListSkuImages",
            post(backoffice::list_sku_images).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/SetSkuImages",
            post(backoffice::set_sku_images).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ListDigitalAssets",
            post(backoffice::list_digital_assets).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateDigitalAsset",
            post(backoffice::create_digital_asset).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateDigitalUploadUrl",
            post(backoffice::create_digital_upload_url).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CatalogWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateDigitalDownloadUrl",
            post(backoffice::create_digital_download_url).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CatalogRead,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/SetInventory",
            post(backoffice::set_inventory).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CatalogWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/ListOrders",
            post(backoffice::list_orders).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::OrdersRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/UpdateOrderStatus",
            post(backoffice::update_order_status).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::OrdersWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreateShipment",
            post(backoffice::create_shipment).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::OrdersWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/UpdateShipmentStatus",
            post(backoffice::update_shipment_status).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::OrdersWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/CreatePromotion",
            post(backoffice::create_promotion).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::PromotionsWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.BackofficeService/UpdatePromotion",
            post(backoffice::update_promotion).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::PromotionsWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/ListCustomers",
            post(customer::list_customers).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CustomersRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/GetCustomer",
            post(customer::get_customer).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CustomersRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/CreateCustomer",
            post(customer::create_customer).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CustomersWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/UpdateCustomer",
            post(customer::update_customer).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::CustomersWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/UpsertCustomerIdentity",
            post(customer::upsert_customer_identity).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CustomersWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/UpsertCustomerAddress",
            post(customer::upsert_customer_address).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CustomersWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/ListCustomerMetafieldDefinitions",
            post(customer::list_customer_metafield_definitions).route_layer(middleware::from_fn_with_state(
                state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CustomersRead,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/CreateCustomerMetafieldDefinition",
            post(customer::create_customer_metafield_definition).route_layer(middleware::from_fn_with_state(
                state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CustomersWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/UpdateCustomerMetafieldDefinition",
            post(customer::update_customer_metafield_definition).route_layer(middleware::from_fn_with_state(
                state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CustomersWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/ListCustomerMetafieldValues",
            post(customer::list_customer_metafield_values).route_layer(middleware::from_fn_with_state(
                state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CustomersRead,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.CustomerService/UpsertCustomerMetafieldValue",
            post(customer::upsert_customer_metafield_value).route_layer(middleware::from_fn_with_state(
                state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::CustomersWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/GetStoreSettings",
            post(store_settings::get_store_settings).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsRead,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/UpdateStoreSettings",
            post(store_settings::update_store_settings).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/InitializeStoreSettings",
            post(store_settings::initialize_store_settings).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/GetMallSettings",
            post(store_settings::get_mall_settings).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsRead,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/UpdateMallSettings",
            post(store_settings::update_mall_settings).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/ListStoreLocations",
            post(store_settings::list_store_locations).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsRead,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/UpsertStoreLocation",
            post(store_settings::upsert_store_location).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/DeleteStoreLocation",
            post(store_settings::delete_store_location).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/ListShippingZones",
            post(store_settings::list_shipping_zones).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsRead,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/UpsertShippingZone",
            post(store_settings::upsert_shipping_zone).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/DeleteShippingZone",
            post(store_settings::delete_shipping_zone).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/ListShippingRates",
            post(store_settings::list_shipping_rates).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsRead,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/UpsertShippingRate",
            post(store_settings::upsert_shipping_rate).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/DeleteShippingRate",
            post(store_settings::delete_shipping_rate).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::SettingsWrite,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/ListTaxRules",
            post(store_settings::list_tax_rules).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::SettingsRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/UpsertTaxRule",
            post(store_settings::upsert_tax_rule).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::SettingsWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.StoreSettingsService/DeleteTaxRule",
            post(store_settings::delete_tax_rule).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::SettingsWrite,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.SetupService/InitializeStore",
            post(setup::initialize_store),
        )
        .route(
            "/rpc/ecommerce.v1.SetupService/ValidateStoreCode",
            post(setup::validate_store_code),
        )
        // IdentityService is the single entry point for staff auth/roles.
        .route(
            "/rpc/ecommerce.v1.IdentityService/SignIn",
            post(identity::sign_in),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/SignOut",
            post(identity::sign_out),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/RefreshToken",
            post(identity::refresh_token),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/ListMyPermissions",
            post(identity::list_my_permissions),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/CreateStaff",
            post(identity::create_staff).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/CreateRole",
            post(identity::create_role).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/AssignRoleToStaff",
            post(identity::assign_role_to_staff).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/ListRoles",
            post(identity::list_roles).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/ListRolesWithPermissions",
            post(identity::list_roles_with_permissions).route_layer(middleware::from_fn_with_state(state.clone(),
                |State(state), req, next| {
                    permissions::require_permission_key(
                        state,
                        req,
                        next,
                        permissions::PermissionKey::StaffManage,
                    )
                },
            )),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/UpdateRole",
            post(identity::update_role).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/DeleteRole",
            post(identity::delete_role).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/ListStaff",
            post(identity::list_staff).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/ListStaffSessions",
            post(identity::list_staff_sessions).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/ForceSignOutStaff",
            post(identity::force_sign_out_staff).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/UpdateStaff",
            post(identity::update_staff).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/InviteStaff",
            post(identity::invite_staff).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/AcceptInvite",
            post(identity::accept_invite),
        )
        .route(
            "/rpc/ecommerce.v1.IdentityService/TransferOwner",
            post(identity::transfer_owner).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::StaffManage,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuditService/ListAuditLogs",
            post(audit::list_audit_logs).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuditRead,
                )
            })),
        )
        .route(
            "/rpc/ecommerce.v1.AuditService/ListAuditActions",
            post(audit::list_audit_actions).route_layer(middleware::from_fn_with_state(state.clone(), |State(state), req, next| {
                permissions::require_permission_key(
                    state,
                    req,
                    next,
                    permissions::PermissionKey::AuditRead,
                )
            })),
        )
        .layer(middleware::from_fn(
            request_context::inject_request_context,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session::require_active_staff_session,
        ))
        .layer(middleware::from_fn(actor::inject_actor))
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &Request<_>| {
                let request_id = req
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("-");
                info_span!(
                    "http_request",
                    method = %req.method(),
                    uri = %req.uri(),
                    request_id = %request_id,
                    trace_id = tracing::field::Empty
                )
            }),
        )
        .layer(cors)
        .with_state(state)
}
