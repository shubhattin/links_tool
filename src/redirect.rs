//! Short-link redirects (parity with SvelteKit `get_redirect_response` and `[name]` routes).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET` | `/{name}` | [`redirect_by_name`] |
//! | `GET` | `/{name}/{num}` | [`redirect_by_name_num`] |
//!
//! JSON error bodies (HTTP 200, SvelteKit parity): [`wrong_url`], [`link_not_found`], [`link_disabled`].

pub use crate::entities::links::Model as Link;
use crate::state::AppState;
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{HeaderValue, LOCATION};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Serialize;

#[derive(Serialize)]
pub struct DetailBody<'a> {
    pub detail: &'a str,
}

/// Short-link routes mounted at the app root (see [`router`]).
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{name}/{num}", get(redirect_by_name_num)) // GET /{name}/{num}
        .route("/{name}", get(redirect_by_name)) // GET /{name}
}

/// `GET /{name}` — short link without numeric substitution (`{0}` must be absent).
async fn redirect_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if name.is_empty() {
        return wrong_url();
    }
    match crate::db::lookup_link(&state.db, &name).await {
        Ok(Some(row)) => response_name_only(&row),
        Ok(None) => link_not_found(),
        Err(_) => internal_server_error().into_response(),
    }
}

/// `GET /{name}/{num}` — short link with `{0}` replaced by `num` (zero-padded).
async fn redirect_by_name_num(
    State(state): State<AppState>,
    Path((name, num)): Path<(String, String)>,
) -> impl IntoResponse {
    if name.is_empty() {
        return wrong_url();
    }
    let num_f = match num.parse::<f64>() {
        Ok(n) if n.is_finite() => n,
        _ => return wrong_url(),
    };
    match crate::db::lookup_link(&state.db, &name).await {
        Ok(Some(row)) => response_with_num(&row, num_f),
        Ok(None) => link_not_found(),
        Err(_) => internal_server_error().into_response(),
    }
}

fn internal_server_error() -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
}

/// JSON error body with HTTP 200 (matches SvelteKit `JSONResponse`).
fn json_detail(detail: &'static str) -> Response {
    (StatusCode::OK, Json(DetailBody { detail })).into_response()
}

/// `GET /{name}` · `GET /{name}/{num}` — invalid path or `{num}` parse failure.
pub fn wrong_url() -> Response {
    json_detail("Wrong URL")
}

/// `GET /{name}` · `GET /{name}/{num}` — no link row or template mismatch.
pub fn link_not_found() -> Response {
    json_detail("Link Not Found")
}

/// `GET /{name}` · `GET /{name}/{num}` — link exists but `enabled` is false.
pub fn link_disabled() -> Response {
    json_detail("Link Disabled")
}

/// `GET /{name}` — link must not contain `{0}`; substitution uses `num = 0`.
pub fn response_name_only(row: &Link) -> Response {
    if row.link.contains("{0}") {
        return link_not_found();
    }
    build_redirect_response(row, 0.0)
}

/// `GET /{name}/{num}` — link must contain `{0}`.
pub fn response_with_num(row: &Link, num: f64) -> Response {
    if !row.link.contains("{0}") {
        return link_not_found();
    }
    build_redirect_response(row, num)
}

/// `GET /{name}` · `GET /{name}/{num}` — 302 redirect when link is enabled.
fn build_redirect_response(row: &Link, num: f64) -> Response {
    if !row.enabled {
        return link_disabled();
    }
    let replacement = format_substitution(row.prefix_zeros, num);
    let expanded = row.link.replacen("{0}", &replacement, 1);
    // SvelteKit `redirect(302, link)` — use 302 Found, not Axum `Redirect::temporary` (307).
    match HeaderValue::try_from(expanded) {
        Ok(location) => (StatusCode::FOUND, [(LOCATION, location)]).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn format_substitution(prefix_zeros: i32, num: f64) -> String {
    let num_str = num_to_js_string(num);
    let pad_len = (prefix_zeros as isize - num_str.len() as isize).max(0) as usize;
    format!("{}{}", "0".repeat(pad_len), num_str)
}

/// Approximate `Number.prototype.toString()` for substitution padding.
fn num_to_js_string(num: f64) -> String {
    if !num.is_finite() {
        return String::new();
    }
    let i = num as i64;
    if num.fract() == 0.0 && (i as f64) == num {
        format!("{i}")
    } else {
        format!("{num}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitution_matches_padding_rule() {
        assert_eq!(format_substitution(5, 42.0), "00042");
        assert_eq!(format_substitution(0, 42.0), "42");
    }
}
