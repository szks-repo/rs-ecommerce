use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace as sdktrace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(service_name: &str) {
    let registry = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env());

    let use_json = std::env::var("LOG_FORMAT").ok().as_deref() == Some("json");

    if std::env::var("ENABLE_OTEL").ok().as_deref() == Some("true") {
        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string());
        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .with_trace_config(
                sdktrace::Config::default().with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                ])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("otel tracer provider");
        let tracer = provider.tracer(service_name.to_string());
        opentelemetry::global::set_tracer_provider(provider);
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        // tracing-opentelemetry 0.26 expects the JSON formatter.
        registry
            .with(tracing_subscriber::fmt::layer().json())
            .with(otel_layer)
            .init();
    } else if use_json {
        registry
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        registry.with(tracing_subscriber::fmt::layer()).init();
    }
}
