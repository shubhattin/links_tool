//! Auth / admin middleware for protected API routes.
//!
//! Stack as outer → inner: [`insert_auth_info`] then [`require_admin`].
//! `insert_auth_info` verifies the session and inserts [`AuthUser`] (`Arc` inside) into
//! request extensions; `require_admin` only checks `role == admin` from that value.
//! Handlers can take [`AuthUser`] via [`FromRequestParts`] (cheap `Arc` clone).

use super::routes::{AuthUser, AuthUserInner};
use super::{ACCESS_COOKIE_NAME, ADMIN_ROLE, ErrorBody, cookie_value, verify_access_token};
use crate::entities::user;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sea_orm::EntityTrait;
use std::sync::Arc;

fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorBody {
            error: message.into(),
        }),
    )
        .into_response()
}

/// Verify access cookie, load user, reject banned accounts, insert [`AuthUser`] into extensions.
pub async fn insert_auth_info(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(token) = cookie_value(request.headers(), ACCESS_COOKIE_NAME) else {
        return json_error(StatusCode::UNAUTHORIZED, "not authenticated");
    };

    let Ok(claims) = verify_access_token(&state.jwt_secret, &token) else {
        return json_error(StatusCode::UNAUTHORIZED, "invalid token");
    };

    let user = match user::Entity::find_by_id(&claims.sub).one(&state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => return json_error(StatusCode::UNAUTHORIZED, "not authenticated"),
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
    };

    if user.banned {
        return json_error(StatusCode::FORBIDDEN, "account banned");
    }

    request
        .extensions_mut()
        .insert(AuthUser(Arc::new(AuthUserInner {
            user_id: user.id,
            role: user.role,
        })));

    next.run(request).await
}

/// Require `role == admin`. Expects [`insert_auth_info`] to have run first.
pub async fn require_admin(request: Request, next: Next) -> Response {
    let Some(user) = request.extensions().get::<AuthUser>() else {
        return json_error(StatusCode::UNAUTHORIZED, "not authenticated");
    };

    if user.role != ADMIN_ROLE {
        return json_error(StatusCode::FORBIDDEN, "admin access required");
    }

    next.run(request).await
}

/// Attach `insert_auth_info` + `require_admin` when `$state` is `Some`; otherwise return `$router` as-is.
///
/// ### Arguments
/// - `$router` — base router expression (`OpenApiRouter<AppState>` / anything with `.route_layer`)
/// - `$state` — `Option<&AppState>`: `Some` applies the auth stack, `None` leaves `$router` unchanged
///
/// # Example
/// ```ignore
/// with_require_admin!(crate::routes::links::openapi_router(), state)
/// ```
#[macro_export]
macro_rules! with_require_admin {
    ($router:expr, $state:expr) => {{
        let router = $router;
        match $state {
            Some(state) => router
                .route_layer(::axum::middleware::from_fn($crate::auth::require_admin))
                .route_layer(::axum::middleware::from_fn_with_state(
                    state.clone(),
                    $crate::auth::insert_auth_info,
                )),
            None => router,
        }
    }};
}
// Uses `$crate` / `::axum` so call sites do not need those imports.
// Layer order is reverse of execution (last added runs first):
// outer `insert_auth_info` → inner `require_admin` → handler.
