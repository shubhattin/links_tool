use axum::Router;
use axum::extract::{Path, State};
use axum::http::Method;
use axum::http::StatusCode;
use axum::http::Uri;
use axum::http::header::HeaderValue;
use axum::response::IntoResponse;
use axum::routing::get;
use links_tool::db::DbPool;
use std::env;
use std::io;
use tower::ServiceBuilder;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

fn io_error(msg: impl Into<String>) -> Error {
    Box::new(io::Error::other(msg.into()))
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
        return links_tool::redirect::wrong_url();
    }
    let pool = pool.clone();
    match tokio::task::spawn_blocking(move || links_tool::db::lookup_link(&pool, &name)).await {
        Ok(Ok(Some(row))) => links_tool::redirect::response_name_only(&row),
        Ok(Ok(None)) => links_tool::redirect::link_not_found(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("database error: {e}"),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("task join error: {e}"),
        )
            .into_response(),
    }
}

async fn redirect_by_name_num(
    State(pool): State<DbPool>,
    Path((name, num)): Path<(String, String)>,
) -> impl IntoResponse {
    if name.is_empty() {
        return links_tool::redirect::wrong_url();
    }
    let num_f = match num.parse::<f64>() {
        Ok(n) if n.is_finite() => n,
        _ => return links_tool::redirect::wrong_url(),
    };
    let pool = pool.clone();
    match tokio::task::spawn_blocking(move || links_tool::db::lookup_link(&pool, &name)).await {
        Ok(Ok(Some(row))) => links_tool::redirect::response_with_num(&row, num_f),
        Ok(Ok(None)) => links_tool::redirect::link_not_found(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("database error: {e}"),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("task join error: {e}"),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename(".env.local");

    let database_url =
        env::var("PG_DATABASE_URL").map_err(|_| io_error("PG_DATABASE_URL must be set"))?;
    let pool = links_tool::db::build_pool(&database_url).map_err(|e| {
        io_error(format!("failed to create database pool: {e}"))
    })?;

    let cors = env::var("FRONTEND_URL")
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
        .unwrap_or_default();

    let router = Router::new()
        .route("/{name}/{num}", get(redirect_by_name_num))
        .route("/{name}", get(redirect_by_name))
        .fallback(fallback)
        .with_state(pool)
        .layer(cors);

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);
    vercel_runtime::run(app).await
}
