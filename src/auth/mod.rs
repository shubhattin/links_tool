//! Email/password auth with Argon2id, JWT access tokens, and rotating refresh cookies.

mod cookies;
mod jwt;
mod password;
mod routes;
mod session_token;

pub use cookies::{
    ACCESS_COOKIE_NAME, REFRESH_COOKIE_NAME, append_access_cookie, append_refresh_cookie,
    clear_auth_cookies, cookie_value, cookie_value_from_parts,
};
pub use jwt::{AccessClaims, issue_access_token, verify_access_token};
pub use routes::{
    ErrorBody, SessionResponse, SignInBody, SignUpBody, UserDto, load_jwt_secret, openapi_router,
    router,
};
pub use session_token::issue_refresh_session;

/// Short-lived access token lifetime (seconds).
pub const ACCESS_TOKEN_TTL_SECS: i64 = 15 * 60;

/// Refresh session lifetime (seconds).
pub const REFRESH_TOKEN_TTL_SECS: i64 = 7 * 24 * 60 * 60;

/// Provider id for email/password accounts (OAuth-ready schema).
pub const CREDENTIAL_PROVIDER: &str = "credential";

/// Default role for newly registered users.
pub const DEFAULT_USER_ROLE: &str = "user";
