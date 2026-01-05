use axum::http::HeaderValue;
use axum::{
    body::Body,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use opentelemetry::trace::{TraceContextExt, TraceId};
use tracing::info;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Clone, Default)]
pub struct RequestContext {
    pub request_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub store_id: Option<String>,
    pub tenant_id: Option<String>,
}

tokio::task_local! {
    static REQUEST_CONTEXT: RequestContext;
}

pub async fn inject_request_context(req: Request<Body>, next: Next) -> Response {
    let auth_ctx = req
        .extensions()
        .get::<Option<crate::rpc::actor::AuthContext>>()
        .and_then(|v| v.clone());
    let ctx = RequestContext {
        request_id: extract_request_id(req.headers()),
        ip_address: extract_ip_address(req.headers()),
        user_agent: extract_user_agent(req.headers()),
        store_id: auth_ctx.as_ref().and_then(|ctx| ctx.store_id.clone()),
        tenant_id: auth_ctx.as_ref().and_then(|ctx| ctx.tenant_id.clone()),
    };
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let request_id = ctx.request_id.clone();
    // Attach trace_id to the current span if available (OpenTelemetry).
    if let Some(trace_id) = current_trace_id().or_else(|| request_id.clone()) {
        tracing::Span::current().record("trace_id", &trace_id);
    }
    let mut res = REQUEST_CONTEXT
        .scope(ctx.clone(), async move {
            info!(
                request_id = request_id.as_deref().unwrap_or(""),
                method = %method,
                path = %path,
                "request start"
            );
            next.run(req).await
        })
        .await;
    info!(
        request_id = ctx.request_id.as_deref().unwrap_or(""),
        status = %res.status(),
        "request end"
    );
    attach_request_id(&mut res, ctx.request_id);
    res
}

fn attach_request_id(res: &mut Response, request_id: Option<String>) {
    let Some(request_id) = request_id else { return };
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        res.headers_mut().insert("x-request-id", value);
    }
}

pub fn current() -> Option<RequestContext> {
    REQUEST_CONTEXT.try_with(|ctx| ctx.clone()).ok()
}

fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers.get("x-request-id")
        && let Ok(value) = value.to_str()
            && !value.is_empty() {
                return Some(value.to_string());
            }
    Some(uuid::Uuid::new_v4().to_string())
}

fn extract_ip_address(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers.get("x-forwarded-for")
        && let Ok(value) = value.to_str()
            && let Some(first) = value.split(',').next() {
                let first = first.trim();
                if !first.is_empty() {
                    return Some(first.to_string());
                }
            }
    if let Some(value) = headers.get("x-real-ip")
        && let Ok(value) = value.to_str()
            && !value.is_empty() {
                return Some(value.to_string());
            }
    None
}

fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
}

fn current_trace_id() -> Option<String> {
    let span = tracing::Span::current();
    let ctx = span.context();
    let trace_id = ctx.span().span_context().trace_id();
    if trace_id == TraceId::INVALID {
        None
    } else {
        Some(trace_id.to_string())
    }
}
