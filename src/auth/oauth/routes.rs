//! OAuth HTTP routes (mounted under `/api/auth`).

use crate::auth::oauth::config::load_oauth_env;
use crate::auth::oauth::github::{finish_github, start_github_async};
use crate::auth::oauth::google::{finish_google, start_google_async};
use crate::auth::oauth::link::{LinkError, find_or_link_user};
use crate::auth::oauth::state_cookie::{
    append_pending_cookie, clear_pending_cookie, read_pending_cookie,
};
use crate::auth::session_issue::{apply_session_cookies, issue_auth_session};
use crate::state::AppState;
use axum::extract::Query;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

fn redirect_frontend(env: &crate::auth::oauth::config::OAuthEnv, query: &str) -> Response {
    let url = if query.is_empty() {
        format!("{}/", env.frontend_url)
    } else {
        format!("{}/?{}", env.frontend_url, query)
    };
    Redirect::temporary(&url).into_response()
}

fn redirect_frontend_error(env: &crate::auth::oauth::config::OAuthEnv, message: &str) -> Response {
    let encoded = urlencoding::encode(message);
    let mut response = redirect_frontend(env, &format!("error={encoded}"));
    clear_pending_cookie(&mut response);
    response
}

async fn oauth_success_redirect(
    state: &AppState,
    env: &crate::auth::oauth::config::OAuthEnv,
    user: &crate::entities::user::Model,
) -> Response {
    match issue_auth_session(state, user).await {
        Ok(session) => {
            let mut response = redirect_frontend(env, "");
            apply_session_cookies(&mut response, &session);
            clear_pending_cookie(&mut response);
            response
        }
        Err(_) => redirect_frontend_error(env, "session_error"),
    }
}

/// `GET /api/auth/google` — redirect to Google OAuth.
#[utoipa::path(
    get,
    path = "/google",
    operation_id = "auth.google",
    tag = "auth",
    responses(
        (status = 302, description = "Redirect to Google authorization"),
        (status = 500, description = "OAuth configuration error"),
    )
)]
pub async fn google_start(State(state): State<AppState>) -> Response {
    let env = match load_oauth_env() {
        Ok(e) => e,
        Err(msg) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
        }
    };

    let start = match start_google_async(&env).await {
        Ok(s) => s,
        Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    };

    let mut response = Redirect::temporary(&start.authorize_url).into_response();
    append_pending_cookie(&mut response, &state.jwt_secret, &start.pending);
    response
}

/// `GET /api/auth/callback/google` — Google OAuth callback.
#[utoipa::path(
    get,
    path = "/callback/google",
    operation_id = "auth.callbackGoogle",
    tag = "auth",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "CSRF state"),
        ("error" = Option<String>, Query, description = "Provider error"),
    ),
    responses(
        (status = 302, description = "Redirect to frontend with session cookies or error"),
    )
)]
pub async fn google_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<OAuthCallbackQuery>,
) -> Response {
    let env = match load_oauth_env() {
        Ok(e) => e,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Some(err) = query.error {
        let msg = query.error_description.unwrap_or(err);
        return redirect_frontend_error(&env, &msg);
    }

    let Some(code) = query.code.filter(|c| !c.is_empty()) else {
        return redirect_frontend_error(&env, "missing_code");
    };
    let Some(returned_state) = query.state.filter(|s| !s.is_empty()) else {
        return redirect_frontend_error(&env, "missing_state");
    };

    let Some(pending) = read_pending_cookie(&headers, &state.jwt_secret) else {
        return redirect_frontend_error(&env, "invalid_oauth_session");
    };

    let profile = match finish_google(&env, &pending, &code, &returned_state).await {
        Ok(p) => p,
        Err(msg) => return redirect_frontend_error(&env, &msg),
    };

    match find_or_link_user(&state, profile).await {
        Ok(user) => oauth_success_redirect(&state, &env, &user).await,
        Err(LinkError::Banned) => redirect_frontend_error(&env, "account_banned"),
        Err(LinkError::EmailRequired) => redirect_frontend_error(&env, "email_required"),
        Err(LinkError::EmailConflict) => redirect_frontend_error(&env, "email_conflict"),
        Err(LinkError::Db(_)) => redirect_frontend_error(&env, "database_error"),
    }
}

/// `GET /api/auth/github` — redirect to GitHub OAuth.
#[utoipa::path(
    get,
    path = "/github",
    operation_id = "auth.github",
    tag = "auth",
    responses(
        (status = 302, description = "Redirect to GitHub authorization"),
        (status = 500, description = "OAuth configuration error"),
    )
)]
pub async fn github_start(State(state): State<AppState>) -> Response {
    let env = match load_oauth_env() {
        Ok(e) => e,
        Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    };

    let start = match start_github_async(&env).await {
        Ok(s) => s,
        Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    };

    let mut response = Redirect::temporary(&start.authorize_url).into_response();
    append_pending_cookie(&mut response, &state.jwt_secret, &start.pending);
    response
}

/// `GET /api/auth/callback/github` — GitHub OAuth callback.
#[utoipa::path(
    get,
    path = "/callback/github",
    operation_id = "auth.callbackGithub",
    tag = "auth",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "CSRF state"),
        ("error" = Option<String>, Query, description = "Provider error"),
    ),
    responses(
        (status = 302, description = "Redirect to frontend with session cookies or error"),
    )
)]
pub async fn github_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<OAuthCallbackQuery>,
) -> Response {
    let env = match load_oauth_env() {
        Ok(e) => e,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Some(err) = query.error {
        let msg = query.error_description.unwrap_or(err);
        return redirect_frontend_error(&env, &msg);
    }

    let Some(code) = query.code.filter(|c| !c.is_empty()) else {
        return redirect_frontend_error(&env, "missing_code");
    };
    let Some(returned_state) = query.state.filter(|s| !s.is_empty()) else {
        return redirect_frontend_error(&env, "missing_state");
    };

    let Some(pending) = read_pending_cookie(&headers, &state.jwt_secret) else {
        return redirect_frontend_error(&env, "invalid_oauth_session");
    };

    let profile = match finish_github(&env, &pending, &code, &returned_state).await {
        Ok(p) => p,
        Err(msg) => return redirect_frontend_error(&env, &msg),
    };

    match find_or_link_user(&state, profile).await {
        Ok(user) => oauth_success_redirect(&state, &env, &user).await,
        Err(LinkError::Banned) => redirect_frontend_error(&env, "account_banned"),
        Err(LinkError::EmailRequired) => redirect_frontend_error(&env, "email_required"),
        Err(LinkError::EmailConflict) => redirect_frontend_error(&env, "email_conflict"),
        Err(LinkError::Db(_)) => redirect_frontend_error(&env, "database_error"),
    }
}

pub fn openapi_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(google_start))
        .routes(routes!(google_callback))
        .routes(routes!(github_start))
        .routes(routes!(github_callback))
}
