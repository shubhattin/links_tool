//! OpenAPI document composition (paths collected via [`utoipa_axum::OpenApiRouter`] in route modules).

use utoipa::Modify;
use utoipa::OpenApi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

use crate::auth::{ErrorBody, SessionResponse, SignInBody, SignUpBody, UserDto};
use crate::redirect::DetailResponse;
use crate::routes::links::{CreateLinkBody, LinkDto, LinksListResponse, UpdateLinkBody};

/// Root OpenAPI document; operation paths are registered by nested [`utoipa_axum::OpenApiRouter`]s.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Links Tool API",
        version = "0.1.0",
        description = "Short-link redirects, link catalog, and cookie-based authentication."
    ),
    components(schemas(
        SignUpBody,
        SignInBody,
        UserDto,
        SessionResponse,
        ErrorBody,
        DetailResponse,
        LinkDto,
        LinksListResponse,
        CreateLinkBody,
        UpdateLinkBody,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Email/password and OAuth social auth (HttpOnly cookies)"),
        (name = "links", description = "Short-link catalog (authenticated)"),
        (name = "redirect", description = "Short-link redirects (SvelteKit parity error bodies)")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "access_cookie",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("access_token"))),
            );
            components.add_security_scheme(
                "refresh_cookie",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("refresh_token"))),
            );
        }
    }
}

/// Build the merged OpenAPI spec (used by `gen-openapi` and optional runtime export).
pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_, api) = crate::app::openapi_router().split_for_parts();
    api
}
