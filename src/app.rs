use crate::db::DbPool;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::Method;
use axum::http::StatusCode;
use axum::http::Uri;
use axum::http::header::HeaderValue;
use axum::response::IntoResponse;
use axum::routing::get;
use std::env;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

fn internal_server_error() -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn fallback(uri: Uri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; charset=utf-8",
        )],
        format!("Not found: {}", uri.path()),
    )
        .into_response()
}

async fn redirect_by_name(
    State(pool): State<DbPool>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if name.is_empty() {
        return crate::redirect::wrong_url();
    }
    match crate::db::lookup_link(&pool, &name).await {
        Ok(Some(row)) => crate::redirect::response_name_only(&row),
        Ok(None) => crate::redirect::link_not_found(),
        Err(_) => internal_server_error().into_response(),
    }
}

async fn redirect_by_name_num(
    State(pool): State<DbPool>,
    Path((name, num)): Path<(String, String)>,
) -> impl IntoResponse {
    if name.is_empty() {
        return crate::redirect::wrong_url();
    }
    let num_f = match num.parse::<f64>() {
        Ok(n) if n.is_finite() => n,
        _ => return crate::redirect::wrong_url(),
    };
    match crate::db::lookup_link(&pool, &name).await {
        Ok(Some(row)) => crate::redirect::response_with_num(&row, num_f),
        Ok(None) => crate::redirect::link_not_found(),
        Err(_) => internal_server_error().into_response(),
    }
}

fn cors_layer_from_env() -> CorsLayer {
    env::var("FRONTEND_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|url| {
            let origin = HeaderValue::try_from(url.trim()).ok()?;
            Some(
                CorsLayer::new()
                    .allow_origin(AllowOrigin::exact(origin))
                    .allow_methods([Method::GET, Method::OPTIONS])
                    .allow_headers(AllowHeaders::any()),
            )
        })
        .unwrap_or_default()
}

/// Axum router for redirect API (no Vercel-specific layers).
pub fn router(pool: DbPool) -> Router {
    let cors = cors_layer_from_env();
    Router::new()
        .route("/{name}/{num}", get(redirect_by_name_num))
        .route("/{name}", get(redirect_by_name))
        .fallback(fallback)
        .with_state(pool)
        .layer(cors)
}
