//! Signed short-lived cookie storing OAuth CSRF state and PKCE verifier.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

const COOKIE_NAME: &str = "oauth_ctx";
const MAX_AGE_SECS: i64 = 600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthPending {
    pub state: String,
    pub pkce_verifier: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub exp: i64,
}

pub fn append_pending_cookie(
    response: &mut axum::response::Response,
    secret: &str,
    pending: &OAuthPending,
) {
    if let Some(value) = encode_pending(secret, pending) {
        let secure = crate::auth::cookies::secure_suffix();
        let header_value = format!(
            "{COOKIE_NAME}={value}; HttpOnly; Path=/api/auth; Max-Age={MAX_AGE_SECS}; SameSite=Lax{secure}"
        );
        if let Ok(h) = axum::http::HeaderValue::from_str(&header_value) {
            response
                .headers_mut()
                .append(axum::http::header::SET_COOKIE, h);
        }
    }
}

pub fn clear_pending_cookie(response: &mut axum::response::Response) {
    let secure = crate::auth::cookies::secure_suffix();
    let header_value =
        format!("{COOKIE_NAME}=; HttpOnly; Path=/api/auth; Max-Age=0; SameSite=Lax{secure}");
    if let Ok(h) = axum::http::HeaderValue::from_str(&header_value) {
        response
            .headers_mut()
            .append(axum::http::header::SET_COOKIE, h);
    }
}

pub fn read_pending_cookie(headers: &axum::http::HeaderMap, secret: &str) -> Option<OAuthPending> {
    let raw = crate::auth::cookie_value(headers, COOKIE_NAME)?;
    decode_pending(secret, &raw)
}

fn encode_pending(secret: &str, pending: &OAuthPending) -> Option<String> {
    let payload = serde_json::to_vec(pending).ok()?;
    let payload_b64 =
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload);
    let sig = sign(secret, &payload_b64)?;
    Some(format!("{payload_b64}.{sig}"))
}

fn decode_pending(secret: &str, raw: &str) -> Option<OAuthPending> {
    let (payload_b64, sig) = raw.split_once('.')?;
    let expected = sign(secret, payload_b64)?;
    if !constant_time_eq(sig.as_bytes(), expected.as_bytes()) {
        return None;
    }
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        payload_b64,
    )
    .ok()?;
    let pending: OAuthPending = serde_json::from_slice(&bytes).ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
    if pending.exp < now {
        return None;
    }
    Some(pending)
}

fn sign(secret: &str, payload_b64: &str) -> Option<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(payload_b64.as_bytes());
    let result = mac.finalize().into_bytes();
    Some(hex::encode(result))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

pub fn new_pending(
    provider: &str,
    state: String,
    pkce_verifier: String,
    nonce: Option<String>,
) -> OAuthPending {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64 + MAX_AGE_SECS)
        .unwrap_or(MAX_AGE_SECS);
    OAuthPending {
        state,
        pkce_verifier,
        provider: provider.to_string(),
        nonce,
        exp,
    }
}
