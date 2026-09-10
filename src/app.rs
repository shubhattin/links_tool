use crate::auth::load_jwt_secret;
use crate::auth::{insert_auth_info, require_admin};
use crate::db::DbPool;
use axum::Router;
use axum::http::StatusCode;
use axum::http::Uri;
use axum::http::header::HeaderValue;
use axum::middleware;
use axum::response::IntoResponse;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

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

fn links_router(state: &AppState) -> Router<AppState> {
    crate::routes::links::router()
        // Outer → inner: insert_auth_info inserts AuthUser, then require_admin checks role.
        .route_layer(middleware::from_fn(require_admin))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            insert_auth_info,
        ))
    // ^ as its a layer so the new layer wraps the previous (reverse order for middleware execution)
}

/// Axum router for redirect API and auth (no Vercel-specific layers).
pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api/auth", crate::auth::router())
        .nest("/api/links", links_router(&state))
        .merge(crate::redirect::router())
        .fallback(fallback)
        .with_state(state)
        .layer(cors_layer_from_env())
}
