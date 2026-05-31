//! Links API (mounted at `/api/links` in [`crate::app::openapi_router`]).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET` | `/api/links` | [`list_links`] |

use crate::auth::{AuthUser, ErrorBody};
use crate::db::{DbPool, LookupError};
use crate::entities::links;
pub use crate::entities::links::Model as Link;
use crate::state::AppState;
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::EntityTrait;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Load every short link row and its attributes.
pub async fn list_all(db: &DbPool) -> Result<Vec<Link>, LookupError> {
    links::Entity::find().all(db).await.map_err(LookupError::Db)
}

/// One short link and its stored attributes.
#[derive(Debug, Serialize, ToSchema)]
pub struct LinkDto {
    pub id: String,
    pub enabled: bool,
    pub link: String,
    pub prefix_zeros: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl From<Link> for LinkDto {
    fn from(row: Link) -> Self {
        Self {
            id: row.id,
            enabled: row.enabled,
            link: row.link,
            prefix_zeros: row.prefix_zeros,
            name: row.name,
        }
    }
}

/// `GET /api/links` response body.
#[derive(Debug, Serialize, ToSchema)]
pub struct LinksListResponse {
    pub links: Vec<LinkDto>,
}

/// Links sub-router with OpenAPI path registration; nest at `/api/links`.
pub fn openapi_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_links))
}

/// Links sub-router (Axum only).
pub fn router() -> Router<AppState> {
    openapi_router().into()
}

/// `GET /api/links` — all short links (requires access cookie).
#[utoipa::path(
    get,
    path = "/",
    operation_id = "links.list",
    tag = "links",
    security(("access_cookie" = [])),
    responses(
        (status = 200, description = "All short links", body = LinksListResponse),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_links(State(state): State<AppState>, _user: AuthUser) -> impl IntoResponse {
    match crate::routes::links::list_all(&state.db).await {
        Ok(rows) => Json(LinksListResponse {
            links: rows.into_iter().map(LinkDto::from).collect(),
        })
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
