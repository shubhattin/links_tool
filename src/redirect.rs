//! Short-link redirects (parity with SvelteKit `get_redirect_response` and `[name]` routes).

use crate::schema::links;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::OptionalExtension;
use serde::Serialize;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Link {
    pub id: String,
    pub enabled: bool,
    pub link: String,
    pub prefix_zeros: i32,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct DetailBody {
    pub detail: String,
}

pub fn find_link_by_id(conn: &mut PgConnection, id: &str) -> QueryResult<Option<Link>> {
    links::table
        .filter(links::id.eq(id))
        .select(Link::as_select())
        .first(conn)
        .optional()
}

/// JSON error body with HTTP 200 (matches SvelteKit `JSONResponse`).
fn json_detail(detail: &'static str) -> Response {
    (
        StatusCode::OK,
        Json(DetailBody {
            detail: detail.to_string(),
        }),
    )
        .into_response()
}

pub fn wrong_url() -> Response {
    json_detail("Wrong URL")
}

pub fn link_not_found() -> Response {
    json_detail("Link Not Found")
}

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

fn build_redirect_response(row: &Link, num: f64) -> Response {
    if !row.enabled {
        return link_disabled();
    }
    let replacement = format_substitution(row.prefix_zeros, num);
    let expanded = row.link.replacen("{0}", &replacement, 1);
    Redirect::temporary(&expanded).into_response()
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
