use crate::auth::load_jwt_secret;
use crate::db::DbPool;
use axum::Router;
use axum::http::StatusCode;
use axum::http::Uri;
use axum::http::header::HeaderValue;
use axum::response::IntoResponse;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

pub use crate::state::AppState;

pub fn build_state(db: DbPool) -> Result<crate::state::AppState, String> {
    Ok(AppState {
        db,
        jwt_secret: load_jwt_secret()?,
    })
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

fn cors_layer_from_env() -> CorsLayer {
    std::env::var("FRONTEND_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|url| {
            let origin = HeaderValue::try_from(url.trim()).ok()?;
            Some(
                CorsLayer::new()
                    .allow_origin(AllowOrigin::exact(origin))
                    .allow_methods(AllowMethods::mirror_request())
                    .allow_headers(AllowHeaders::mirror_request())
                    .allow_credentials(true),
            )
        })
        .unwrap_or_default()
}

/// Composed OpenAPI-aware router (auth + redirects).
pub fn openapi_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(crate::openapi::ApiDoc::openapi())
        .nest("/api/auth", crate::auth::openapi_router())
        .merge(crate::redirect::openapi_router())
        .fallback(fallback)
}

/// Axum router for redirect API and auth (no Vercel-specific layers).
pub fn router(state: crate::state::AppState) -> Router {
    let cors = cors_layer_from_env();
    openapi_router().with_state(state).layer(cors).into()
}
