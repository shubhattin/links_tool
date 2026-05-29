//! Shared session issuance (access + refresh cookies) for password and OAuth sign-in.

use crate::auth::{
    append_access_cookie, append_refresh_cookie, issue_access_token, issue_refresh_session,
    session_token::maybe_purge_expired_sessions,
};
use crate::entities::user;
use crate::state::AppState;
use axum::response::Response;

#[derive(Debug)]
pub enum AuthSessionError {
    Db,
    Token,
}

/// Issued cookie values to attach to any response (JSON or redirect).
pub struct IssuedSession {
    pub access_token: String,
    pub refresh_raw: String,
    pub expires_in: i64,
}

pub async fn issue_auth_session(
    state: &AppState,
    user: &user::Model,
) -> Result<IssuedSession, AuthSessionError> {
    let _ = maybe_purge_expired_sessions(&state.db).await;
    let refresh = issue_refresh_session(&state.db, &user.id)
        .await
        .map_err(|_| AuthSessionError::Db)?;
    let (access_token, expires_in) =
        issue_access_token(&state.jwt_secret, &user.id).map_err(|_| AuthSessionError::Token)?;
    Ok(IssuedSession {
        access_token,
        refresh_raw: refresh.raw_token,
        expires_in,
    })
}

pub fn apply_session_cookies(response: &mut Response, session: &IssuedSession) {
    append_access_cookie(response, &session.access_token);
    append_refresh_cookie(response, &session.refresh_raw);
}
