use axum::{
    body::Body,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{collections::HashMap, time::{Duration, Instant}};
use tokio::sync::RwLock;

use crate::pb::pb;

pub async fn inject_actor(mut req: Request<Body>, next: Next) -> Response {
    let actor = actor_from_headers(req.headers()).await;
    req.extensions_mut().insert(actor);
    next.run(req).await
}

async fn actor_from_headers(headers: &HeaderMap) -> Option<pb::ActorContext> {
    if let Some(actor) = actor_from_bearer(headers).await {
        return Some(actor);
    }
    actor_from_override(headers)
}

async fn actor_from_bearer(headers: &HeaderMap) -> Option<pb::ActorContext> {
    let value = headers.get(axum::http::header::AUTHORIZATION)?;
    let value = value.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;
    if token.is_empty() {
        return None;
    }
    if let Some(ctx) = verify_jwt_rs256(token).await {
        return Some(ctx);
    }
    if let Some(ctx) = verify_jwt_hs256(token) {
        return Some(ctx);
    }
    // Fallback: treat bearer token as actor_id in dev (when AUTH_JWT_SECRET is not set).
    if std::env::var("AUTH_JWT_SECRET").is_err() {
        return Some(pb::ActorContext {
            actor_id: token.to_string(),
            actor_type: "api".to_string(),
        });
    }
    None
}

fn actor_from_override(headers: &HeaderMap) -> Option<pb::ActorContext> {
    let actor_id = headers.get("x-actor-id")?.to_str().ok()?.to_string();
    if actor_id.is_empty() {
        return None;
    }
    let actor_type = headers
        .get("x-actor-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("admin")
        .to_string();
    Some(pb::ActorContext { actor_id, actor_type })
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: String,
    #[allow(dead_code)]
    exp: usize,
    actor_type: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Jwks {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize, Clone)]
struct JwkKey {
    kty: String,
    kid: Option<String>,
    alg: Option<String>,
    n: String,
    e: String,
}

#[derive(Default)]
struct JwksCache {
    fetched_at: Option<Instant>,
    keys: HashMap<String, JwkKey>,
}

static JWKS_CACHE: Lazy<RwLock<JwksCache>> = Lazy::new(|| RwLock::new(JwksCache::default()));
const JWKS_TTL: Duration = Duration::from_secs(300);
const JWKS_TIMEOUT: Duration = Duration::from_secs(5);

async fn verify_jwt_rs256(token: &str) -> Option<pb::ActorContext> {
    let jwks_url = std::env::var("AUTH_JWKS_URL").ok()?;
    let header = decode_header(token).ok()?;
    let kid = header.kid;

    let key = get_jwk_key(&jwks_url, kid.as_deref()).await?;
    if key.kty != "RSA" {
        return None;
    }
    if let Some(alg) = &key.alg {
        if alg != "RS256" {
            return None;
        }
    }
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e).ok()?;
    let mut validation = Validation::new(Algorithm::RS256);
    if let Ok(issuer) = std::env::var("AUTH_JWT_ISSUER") {
        validation.set_issuer(&[issuer]);
    }
    if let Ok(aud) = std::env::var("AUTH_JWT_AUDIENCE") {
        validation.set_audience(&[aud]);
    }
    let data = decode::<JwtClaims>(token, &decoding_key, &validation).ok()?;
    Some(pb::ActorContext {
        actor_id: data.claims.sub,
        actor_type: data.claims.actor_type.unwrap_or_else(|| "api".to_string()),
    })
}

fn verify_jwt_hs256(token: &str) -> Option<pb::ActorContext> {
    let secret = std::env::var("AUTH_JWT_SECRET").ok()?;
    let mut validation = Validation::new(Algorithm::HS256);
    if let Ok(issuer) = std::env::var("AUTH_JWT_ISSUER") {
        validation.set_issuer(&[issuer]);
    }
    if let Ok(aud) = std::env::var("AUTH_JWT_AUDIENCE") {
        validation.set_audience(&[aud]);
    }
    let data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .ok()?;
    Some(pb::ActorContext {
        actor_id: data.claims.sub,
        actor_type: data.claims.actor_type.unwrap_or_else(|| "api".to_string()),
    })
}

async fn get_jwk_key(jwks_url: &str, kid: Option<&str>) -> Option<JwkKey> {
    let now = Instant::now();
    {
        let cache = JWKS_CACHE.read().await;
        if let Some(fetched_at) = cache.fetched_at {
            if now.duration_since(fetched_at) < JWKS_TTL {
                if let Some(key) = select_jwk(&cache.keys, kid) {
                    return Some(key.clone());
                }
                // If kid is specified but not found, fall through to refresh.
            }
        }
    }

    let jwks = fetch_jwks(jwks_url).await?;
    let mut keys = HashMap::new();
    for key in jwks.keys.into_iter() {
        let key_id = key.kid.clone().unwrap_or_default();
        keys.insert(key_id, key);
    }

    {
        let mut cache = JWKS_CACHE.write().await;
        cache.fetched_at = Some(now);
        cache.keys = keys;
    }

    let cache = JWKS_CACHE.read().await;
    if let Some(key) = select_jwk(&cache.keys, kid) {
        return Some(key.clone());
    }
    if kid.is_some() && cache.keys.len() == 1 && allow_single_key_fallback() {
        return cache.keys.values().next().cloned();
    }
    None
}

fn select_jwk<'a>(keys: &'a HashMap<String, JwkKey>, kid: Option<&str>) -> Option<&'a JwkKey> {
    if let Some(kid) = kid {
        return keys.get(kid);
    }
    if keys.len() == 1 {
        return keys.values().next();
    }
    None
}

async fn fetch_jwks(jwks_url: &str) -> Option<Jwks> {
    let client = reqwest::Client::builder()
        .timeout(JWKS_TIMEOUT)
        .build()
        .ok()?;
    let resp = client.get(jwks_url).send().await.ok()?;
    let jwks = resp.json::<Jwks>().await.ok()?;
    Some(jwks)
}

fn allow_single_key_fallback() -> bool {
    std::env::var("AUTH_JWKS_FALLBACK_SINGLE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
