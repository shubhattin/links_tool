//! Auth HTTP routes (mounted at `/api/auth` in [`crate::app::router`]).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET`  | `/api/auth/me`       | [`me`] |
//! | `POST` | `/api/auth/sign-up`  | [`sign_up`] |
//! | `POST` | `/api/auth/sign-in`  | [`sign_in`] |
//! | `POST` | `/api/auth/refresh`  | [`refresh`] |
//! | `POST` | `/api/auth/sign-out` | [`sign_out`] |

use crate::state::AppState;
use crate::auth::{
    ACCESS_COOKIE_NAME, CREDENTIAL_PROVIDER, DEFAULT_USER_ROLE, REFRESH_COOKIE_NAME,
    append_access_cookie, append_refresh_cookie, clear_auth_cookies, cookie_value,
    cookie_value_from_parts, issue_access_token, issue_refresh_session, password,
    session_token::{find_valid_session_by_token, revoke_session, rotate_refresh_session},
    verify_access_token,
};
use crate::entities::{account, user};
use axum::Json;
use axum::Router;
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

enum AuthSuccessError {
    Db,
    Token,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AuthUser {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SignUpBody {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub email_verified: bool,
}

/// Cookie-only session response; no JWT in the body.
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

fn normalize_email(email: &str) -> Option<String> {
    let email = email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') || email.len() > 320 {
        return None;
    }
    Some(email)
}

fn validate_password(password: &str) -> bool {
    password.len() >= 8 && password.len() <= 128
}

fn user_dto(u: &user::Model) -> UserDto {
    UserDto {
        id: u.id.clone(),
        email: u.email.clone(),
        name: u.name.clone(),
        role: u.role.clone(),
        email_verified: u.email_verified,
    }
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

async fn load_user_by_id(
    db: &crate::db::DbPool,
    user_id: &str,
) -> Result<Option<user::Model>, sea_orm::DbErr> {
    user::Entity::find_by_id(user_id).one(db).await
}

async fn user_from_access_cookie(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<user::Model, StatusCode> {
    let token = cookie_value(headers, ACCESS_COOKIE_NAME).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims =
        verify_access_token(&state.jwt_secret, &token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = load_user_by_id(&state.db, &claims.sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if user.banned {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(user)
}

async fn auth_success(state: &AppState, user: &user::Model) -> Result<Response, AuthSuccessError> {
    let refresh = issue_refresh_session(&state.db, &user.id)
        .await
        .map_err(|_| AuthSuccessError::Db)?;
    let (access_token, expires_in) =
        issue_access_token(&state.jwt_secret, &user.id).map_err(|_| AuthSuccessError::Token)?;

    let mut response = Json(SessionResponse { expires_in }).into_response();
    append_access_cookie(&mut response, &access_token);
    append_refresh_cookie(&mut response, &refresh.raw_token);
    Ok(response)
}

/// Auth sub-router; nest at `/api/auth` for full paths in the module docs above.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(me)) // GET /api/auth/me
        .route("/sign-up", post(sign_up)) // POST /api/auth/sign-up
        .route("/sign-in", post(sign_in)) // POST /api/auth/sign-in
        .route("/refresh", post(refresh)) // POST /api/auth/refresh
        .route("/sign-out", post(sign_out)) // POST /api/auth/sign-out
}

/// `GET /api/auth/me` — current user from access cookie (no JWT in response body).
async fn me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match user_from_access_cookie(&state, &headers).await {
        Ok(user) => Json(user_dto(&user)).into_response(),
        Err(StatusCode::FORBIDDEN) => json_error(StatusCode::FORBIDDEN, "account banned"),
        Err(_) => json_error(StatusCode::UNAUTHORIZED, "not authenticated"),
    }
}

/// `POST /api/auth/sign-up` — register with email, password, name; sets auth cookies.
async fn sign_up(State(state): State<AppState>, Json(body): Json<SignUpBody>) -> Response {
    let Some(email) = normalize_email(&body.email) else {
        return json_error(StatusCode::BAD_REQUEST, "invalid email");
    };
    if !validate_password(&body.password) {
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
        ..Default::default()
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
async fn sign_in(State(state): State<AppState>, Json(body): Json<SignInBody>) -> Response {
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
async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> Response {
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
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "session error"),
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
async fn sign_out(State(state): State<AppState>, headers: HeaderMap) -> Response {
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
        let token = cookie_value_from_parts(parts, ACCESS_COOKIE_NAME).ok_or_else(|| {
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
