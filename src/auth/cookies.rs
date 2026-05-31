use crate::auth::{ACCESS_TOKEN_TTL_SECS, REFRESH_TOKEN_TTL_SECS};
use axum::http::header::{COOKIE, HeaderMap, HeaderValue, SET_COOKIE};
use axum::response::Response;

pub const ACCESS_COOKIE_NAME: &str = "access_token";
pub const REFRESH_COOKIE_NAME: &str = "refresh_token";

pub(crate) fn secure_suffix() -> &'static str {
    let secure = std::env::var("AUTH_COOKIE_SECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if secure { "; Secure" } else { "" }
}

pub fn append_access_cookie(response: &mut Response, token: &str) {
    let max_age = ACCESS_TOKEN_TTL_SECS;
    let secure = secure_suffix();
    let value = format!(
        "{ACCESS_COOKIE_NAME}={token}; HttpOnly; Path=/api; Max-Age={max_age}; SameSite=Lax{secure}"
    );
    if let Ok(header) = HeaderValue::from_str(&value) {
        response.headers_mut().append(SET_COOKIE, header);
    }
}

pub fn clear_access_cookie(response: &mut Response) {
    let secure = secure_suffix();
    let value =
        format!("{ACCESS_COOKIE_NAME}=; HttpOnly; Path=/api; Max-Age=0; SameSite=Lax{secure}");
    if let Ok(header) = HeaderValue::from_str(&value) {
        response.headers_mut().append(SET_COOKIE, header);
    }
}

pub fn append_refresh_cookie(response: &mut Response, raw_token: &str) {
    let max_age = REFRESH_TOKEN_TTL_SECS;
    let secure = secure_suffix();
    let value = format!(
        "{REFRESH_COOKIE_NAME}={raw_token}; HttpOnly; Path=/api/auth; Max-Age={max_age}; SameSite=Lax{secure}"
    );
    if let Ok(header) = HeaderValue::from_str(&value) {
        response.headers_mut().append(SET_COOKIE, header);
    }
}

pub fn clear_refresh_cookie(response: &mut Response) {
    let secure = secure_suffix();
    let value = format!(
        "{REFRESH_COOKIE_NAME}=; HttpOnly; Path=/api/auth; Max-Age=0; SameSite=Lax{secure}"
    );
    if let Ok(header) = HeaderValue::from_str(&value) {
        response.headers_mut().append(SET_COOKIE, header);
    }
}

pub fn clear_auth_cookies(response: &mut Response) {
    clear_access_cookie(response);
    clear_refresh_cookie(response);
}

pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let header = headers.get(COOKIE)?.to_str().ok()?;
    for pair in header.split(';') {
        let mut kv = pair.trim().splitn(2, '=');
        let key = kv.next()?.trim();
        if key == name {
            return kv.next().map(|v| v.to_string());
        }
    }
    None
}
