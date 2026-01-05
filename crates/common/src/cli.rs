pub fn init(service_name: &str) {
    crate::telemetry::init_tracing(service_name);
}
