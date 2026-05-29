//! Short-link redirects (parity with SvelteKit `get_redirect_response` and `[name]` routes).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET` | `/{name}` | [`redirect_by_name`] |
//! | `GET` | `/{name}/{num}` | [`redirect_by_name_num`] |
//!
//! JSON error bodies (HTTP 200, SvelteKit parity) via [`json_detail`].

pub use crate::entities::links::Model as Link;
use crate::state::AppState;
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{HeaderValue, LOCATION};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// JSON detail body (HTTP 200 for SvelteKit parity errors).
#[derive(Serialize, ToSchema)]
pub struct DetailResponse {
    pub detail: String,
}

/// Short-link routes with OpenAPI path registration (mounted at app root).
pub fn openapi_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(redirect_by_name))
        .routes(routes!(redirect_by_name_num))
}

/// Short-link routes mounted at the app root (see [`openapi_router`]).
pub fn router() -> Router<AppState> {
    openapi_router().into()
}

/// `GET /{name}` — short link without numeric substitution (`{0}` must be absent).
#[utoipa::path(
    get,
    path = "/{name}",
    operation_id = "redirect.byName",
    tag = "redirect",
    params(("name" = String, Path, description = "Short link id")),
    responses(
        (status = 302, description = "Redirect to target URL"),
        (status = 200, description = "Wrong URL, link not found, or disabled", body = DetailResponse),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn redirect_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if name.is_empty() {
        return json_detail("Wrong URL");
    }
    match crate::db::lookup_link(&state.db, &name).await {
        Ok(Some(row)) => {
            if row.link.contains("{0}") {
                json_detail("Link Not Found")
            } else {
                build_redirect_response(&row, 0.0)
            }
        }
        Ok(None) => json_detail("Link Not Found"),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// `GET /{name}/{num}` — short link with `{0}` replaced by `num` (zero-padded).
#[utoipa::path(
    get,
    path = "/{name}/{num}",
    operation_id = "redirect.byNameNum",
    tag = "redirect",
    params(
        ("name" = String, Path, description = "Short link id"),
        ("num" = String, Path, description = "Numeric substitution for `{0}` in template"),
    ),
    responses(
        (status = 302, description = "Redirect to target URL"),
        (status = 200, description = "Wrong URL, link not found, or disabled", body = DetailResponse),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn redirect_by_name_num(
    State(state): State<AppState>,
    Path((name, num)): Path<(String, String)>,
) -> impl IntoResponse {
    if name.is_empty() {
        return json_detail("Wrong URL");
    }
    let num_f = match num.parse::<f64>() {
        Ok(n) if n.is_finite() => n,
        _ => return json_detail("Wrong URL"),
    };
    match crate::db::lookup_link(&state.db, &name).await {
        Ok(Some(row)) => {
            if !row.link.contains("{0}") {
                json_detail("Link Not Found")
            } else {
                build_redirect_response(&row, num_f)
            }
        }
        Ok(None) => json_detail("Link Not Found"),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// JSON error body with HTTP 200 (matches SvelteKit `JSONResponse`).
fn json_detail(detail: &'static str) -> Response {
    (
        StatusCode::OK,
        Json(DetailResponse {
            detail: detail.to_string(),
        }),
    )
        .into_response()
}

/// `GET /{name}` · `GET /{name}/{num}` — 302 redirect when link is enabled.
fn build_redirect_response(row: &Link, num: f64) -> Response {
    if !row.enabled {
        return json_detail("Link Disabled");
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
    let num_str = if !num.is_finite() {
        String::new()
    } else {
        let i = num as i64;
        if num.fract() == 0.0 && (i as f64) == num {
            format!("{i}")
        } else {
            format!("{num}")
        }
    };
    let pad_len = (prefix_zeros as isize - num_str.len() as isize).max(0) as usize;
    format!("{}{}", "0".repeat(pad_len), num_str)
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
