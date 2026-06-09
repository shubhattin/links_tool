//! Admin-only middleware for protected API routes.

use super::{ACCESS_COOKIE_NAME, ADMIN_ROLE, ErrorBody, cookie_value, verify_access_token};
use crate::entities::user;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sea_orm::EntityTrait;

fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorBody {
            error: message.into(),
        }),
    )
        .into_response()
}

/// Require a valid access cookie and `role == admin` before serving the route.
pub async fn require_admin(
    State(state): State<AppState>,
    request: Request,
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

    if user.role != ADMIN_ROLE {
        return json_error(StatusCode::FORBIDDEN, "admin access required");
    }

    next.run(request).await
}
