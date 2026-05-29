//! Google (OIDC) and GitHub (OAuth2) social sign-in.

mod config;
mod github;
mod google;
mod link;
mod routes;
mod state_cookie;

pub use routes::openapi_router;
