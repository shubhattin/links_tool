//! Auth HTTP routes (mounted at `/api/auth` in [`crate::app::router`]).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET`  | `/api/auth/me`       | [`me`] |
//! | `POST` | `/api/auth/sign-up`  | [`sign_up`] |
//! | `POST` | `/api/auth/sign-in`  | [`sign_in`] |
//! | `POST` | `/api/auth/refresh`  | [`refresh`] |
//! | `POST` | `/api/auth/sign-out` | [`sign_out`] |
//! | `GET`  | `/api/auth/google` | OAuth start (Google) |
//! | `GET`  | `/api/auth/callback/google` | OAuth callback (Google) |
//! | `GET`  | `/api/auth/github` | OAuth start (GitHub) |
//! | `GET`  | `/api/auth/callback/github` | OAuth callback (GitHub) |

use crate::auth::{
    ACCESS_COOKIE_NAME, CREDENTIAL_PROVIDER, DEFAULT_USER_ROLE, REFRESH_COOKIE_NAME,
    append_access_cookie, append_refresh_cookie, clear_auth_cookies, cookie_value,
    issue_access_token, normalize_email, password,
    session_issue::{AuthSessionError, apply_session_cookies, issue_auth_session},
    session_token::{
        RotateRefreshError, find_valid_session_by_token, maybe_purge_expired_sessions,
        revoke_session, rotate_refresh_session,
    },
    verify_access_token,
};
use crate::entities::{account, user};
use crate::state::AppState;
use axum::Json;
use axum::Router;
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AuthUser {
    pub user_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignUpBody {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignInBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub email_verified: bool,
    pub is_maintainer: bool,
    pub image: Option<String>,
    pub username: Option<String>,
    pub display_username: Option<String>,
}

/// Cookie-only session response; no JWT in the body.
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    #[schema(value_type = i32, example = 900)]
    pub expires_in: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorBody {
            error: message.into(),
        }),
    )
        .into_response()
}

async fn auth_success(state: &AppState, user: &user::Model) -> Result<Response, AuthSessionError> {
    let session = issue_auth_session(state, user).await?;
    let mut response = Json(SessionResponse {
        expires_in: session.expires_in,
    })
    .into_response();
    apply_session_cookies(&mut response, &session);
    Ok(response)
}

/// Auth sub-router with OpenAPI path registration; nest at `/api/auth`.
pub fn openapi_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(me))
        .routes(routes!(sign_up))
        .routes(routes!(sign_in))
        .routes(routes!(refresh))
        .routes(routes!(sign_out))
        .merge(crate::auth::oauth::openapi_router())
}

/// Auth sub-router (Axum only).
pub fn router() -> Router<AppState> {
    openapi_router().into()
}

/// `GET /api/auth/me` — current user from access cookie (no JWT in response body).
#[utoipa::path(
    get,
    path = "/me",
    operation_id = "auth.me",
    tag = "auth",
    security(("access_cookie" = [])),
    responses(
        (status = 200, description = "Current user", body = UserDto),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Account banned", body = ErrorBody),
        (status = 500, description = "Internal error", body = ErrorBody),
    )
)]
pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    // Read and verify access cookie JWT.
    let Some(token) = cookie_value(&headers, ACCESS_COOKIE_NAME) else {
        return json_error(StatusCode::UNAUTHORIZED, "not authenticated");
    };
    let Ok(claims) = verify_access_token(&state.jwt_secret, &token) else {
        return json_error(StatusCode::UNAUTHORIZED, "not authenticated");
    };
    // Load user row for claims.sub.
    let user = match user::Entity::find_by_id(&claims.sub).one(&state.db).await {
        Ok(Some(u)) => u,
        Ok(None) => return json_error(StatusCode::UNAUTHORIZED, "not authenticated"),
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
    };
    if user.banned {
        return json_error(StatusCode::FORBIDDEN, "account banned");
    }
    Json(UserDto {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
        email_verified: user.email_verified,
        is_maintainer: user.is_maintainer,
        username: user.username,
        image: user.image,
        display_username: user.display_username,
    })
    .into_response()
}

/// `POST /api/auth/sign-up` — register with email, password, name; sets auth cookies.
#[utoipa::path(
    post,
    path = "/sign-up",
    operation_id = "auth.signUp",
    tag = "auth",
    request_body = SignUpBody,
    responses(
        (status = 200, description = "Session created; Set-Cookie headers set", body = SessionResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 409, description = "Email already registered", body = ErrorBody),
        (status = 500, description = "Internal error", body = ErrorBody),
    )
)]
pub async fn sign_up(State(state): State<AppState>, Json(body): Json<SignUpBody>) -> Response {
    let Some(email) = normalize_email(&body.email) else {
        return json_error(StatusCode::BAD_REQUEST, "invalid email");
    };
    if body.password.len() < 8 || body.password.len() > 128 {
        return json_error(StatusCode::BAD_REQUEST, "password must be 8-128 characters");
    }
    let name = body.name.trim();
    if name.is_empty() || name.len() > 255 {
        return json_error(StatusCode::BAD_REQUEST, "invalid name");
    }

    let existing = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&state.db)
        .await;
    let Ok(existing) = existing else {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error");
    };
    if existing.is_some() {
        return json_error(StatusCode::CONFLICT, "email already registered");
    }

    let password_hash = match password::hash_password(&body.password) {
        Ok(h) => h,
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "hash failed"),
    };

    let user_id = uuid::Uuid::new_v4().to_string();
    let account_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();

    let txn = match state.db.begin().await {
        Ok(t) => t,
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error"),
    };

    let user_model = user::ActiveModel {
        id: Set(user_id.clone()),
        name: Set(name.to_string()),
        email: Set(email.clone()),
        email_verified: Set(false),
        image: Set(None),
        role: Set(DEFAULT_USER_ROLE.to_string()),
        banned: Set(false),
        ban_reason: Set(None),
        ban_expires: Set(None),
        username: Set(None),
        display_username: Set(None),
        is_maintainer: Set(false),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    if user_model.insert(&txn).await.is_err() {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error");
    }

    let account_model = account::ActiveModel {
        id: Set(account_id),
        account_id: Set(email.clone()),
        provider_id: Set(CREDENTIAL_PROVIDER.to_string()),
        user_id: Set(user_id.clone()),
        password: Set(Some(password_hash)),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };
    if account_model.insert(&txn).await.is_err() {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error");
    }

    if txn.commit().await.is_err() {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error");
    }

    let user = match user::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(u)) => u,
        _ => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error"),
    };

    match auth_success(&state, &user).await {
        Ok(r) => r,
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "session error"),
    }
}

/// `POST /api/auth/sign-in` — login; sets auth cookies.
#[utoipa::path(
    post,
    path = "/sign-in",
    operation_id = "auth.signIn",
    tag = "auth",
    request_body = SignInBody,
    responses(
        (status = 200, description = "Session created; Set-Cookie headers set", body = SessionResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Invalid credentials", body = ErrorBody),
        (status = 403, description = "Account banned", body = ErrorBody),
        (status = 500, description = "Internal error", body = ErrorBody),
    )
)]
pub async fn sign_in(State(state): State<AppState>, Json(body): Json<SignInBody>) -> Response {
    let Some(email) = normalize_email(&body.email) else {
        return json_error(StatusCode::BAD_REQUEST, "invalid email");
    };

    let user = match user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return json_error(StatusCode::UNAUTHORIZED, "invalid credentials"),
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error"),
    };

    if user.banned {
        return json_error(StatusCode::FORBIDDEN, "account banned");
    }

    let cred = match account::Entity::find()
        .filter(account::Column::UserId.eq(user.id.clone()))
        .filter(account::Column::ProviderId.eq(CREDENTIAL_PROVIDER))
        .one(&state.db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => return json_error(StatusCode::UNAUTHORIZED, "invalid credentials"),
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error"),
    };

    let Some(hash) = cred.password.as_deref() else {
        return json_error(StatusCode::UNAUTHORIZED, "invalid credentials");
    };
    if !password::verify_password(&body.password, hash) {
        return json_error(StatusCode::UNAUTHORIZED, "invalid credentials");
    }

    match auth_success(&state, &user).await {
        Ok(r) => r,
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "session error"),
    }
}

/// `POST /api/auth/refresh` — rotate refresh session; new access + refresh cookies.
#[utoipa::path(
    post,
    path = "/refresh",
    operation_id = "auth.refresh",
    tag = "auth",
    security(("refresh_cookie" = [])),
    responses(
        (status = 200, description = "Session rotated; Set-Cookie headers set", body = SessionResponse),
        (status = 401, description = "Invalid or missing refresh token", body = ErrorBody),
        (status = 403, description = "Account banned", body = ErrorBody),
        (status = 500, description = "Internal error", body = ErrorBody),
    )
)]
pub async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let _ = maybe_purge_expired_sessions(&state.db).await;
    let raw = match cookie_value(&headers, REFRESH_COOKIE_NAME) {
        Some(t) => t,
        None => return json_error(StatusCode::UNAUTHORIZED, "missing refresh token"),
    };

    let session = match find_valid_session_by_token(&state.db, &raw).await {
        Ok(Some(s)) => s,
        Ok(None) => return json_error(StatusCode::UNAUTHORIZED, "invalid refresh token"),
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error"),
    };

    let user = match user::Entity::find_by_id(session.user_id.clone())
        .one(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return json_error(StatusCode::UNAUTHORIZED, "user not found"),
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error"),
    };

    if user.banned {
        return json_error(StatusCode::FORBIDDEN, "account banned");
    }

    let rotated = match rotate_refresh_session(&state.db, &session.id, &user.id).await {
        Ok(r) => r,
        Err(RotateRefreshError::SessionAlreadyRotated) => {
            return json_error(StatusCode::UNAUTHORIZED, "invalid refresh token");
        }
        Err(RotateRefreshError::Db(_)) => {
            return json_error(StatusCode::INTERNAL_SERVER_ERROR, "session error");
        }
    };

    let (access_token, expires_in) = match issue_access_token(&state.jwt_secret, &user.id) {
        Ok(t) => t,
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "token error"),
    };

    let mut response = Json(SessionResponse { expires_in }).into_response();
    append_access_cookie(&mut response, &access_token);
    append_refresh_cookie(&mut response, &rotated.raw_token);
    response
}

/// `POST /api/auth/sign-out` — revoke session and clear auth cookies.
#[utoipa::path(
    post,
    path = "/sign-out",
    operation_id = "auth.signOut",
    tag = "auth",
    security(("refresh_cookie" = [])),
    responses(
        (status = 204, description = "Signed out; cookies cleared"),
        (status = 500, description = "Internal error", body = ErrorBody),
    )
)]
pub async fn sign_out(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    if let Some(raw) = cookie_value(&headers, REFRESH_COOKIE_NAME)
        && let Ok(Some(session)) = find_valid_session_by_token(&state.db, &raw).await
    {
        let _ = revoke_session(&state.db, &session.id).await;
    }
    clear_auth_cookies(&mut response);
    response
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app = AppState::from_ref(state);
        let token = cookie_value(&parts.headers, ACCESS_COOKIE_NAME).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "not authenticated".into(),
                }),
            )
        })?;

        let claims = verify_access_token(&app.jwt_secret, &token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "invalid token".into(),
                }),
            )
        })?;

        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}

pub fn load_jwt_secret() -> Result<Arc<str>, String> {
    std::env::var("JWT_SECRET")
        .map(Arc::from)
        .map_err(|_| "JWT_SECRET must be set: missing environment variable".to_string())
}
