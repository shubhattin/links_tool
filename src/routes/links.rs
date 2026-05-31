//! Links API (mounted at `/api/links` in [`crate::app::openapi_router`]).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET` | `/api/links` | [`list_links`] |
//! | `POST` | `/api/links` | [`create_link`] |
//! | `PATCH` | `/api/links/{id}` | [`update_link`] |
//! | `DELETE` | `/api/links/{id}` | [`delete_link`] |

use crate::auth::{AuthUser, ErrorBody};
use crate::db::LookupError;
use crate::entities::links;
pub use crate::entities::links::Model as Link;
use crate::state::AppState;
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLinkBody {
    pub id: String,
    pub enabled: bool,
    pub link: String,
    pub prefix_zeros: i32,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLinkBody {
    pub enabled: bool,
    pub link: String,
    pub prefix_zeros: i32,
    pub name: Option<String>,
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

fn validate_id(id: &str) -> Option<Response> {
    let id = id.trim();
    if id.is_empty() || id.len() > 20 {
        return Some(json_error(
            StatusCode::BAD_REQUEST,
            "id must be 1–20 characters",
        ));
    }
    None
}

fn validate_name(name: &Option<String>) -> Option<Response> {
    if let Some(name) = name
        && name.len() > 30 {
            return Some(json_error(
                StatusCode::BAD_REQUEST,
                "name must be at most 30 characters",
            ));
        }
    None
}

fn validate_link(link: &str) -> Option<Response> {
    if link.trim().is_empty() {
        return Some(json_error(StatusCode::BAD_REQUEST, "link is required"));
    }
    None
}

/// Links sub-router with OpenAPI path registration; nest at `/api/links`.
pub fn openapi_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_links, create_link, update_link, delete_link))
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
    match links::Entity::find()
        .all(&state.db)
        .await
        .map_err(LookupError::Db)
    {
        Ok(rows) => Json(LinksListResponse {
            links: rows.into_iter().map(LinkDto::from).collect::<Vec<_>>(),
        })
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// `POST /api/links` — create a short link.
#[utoipa::path(
    post,
    path = "/",
    operation_id = "links.create",
    tag = "links",
    security(("access_cookie" = [])),
    request_body = CreateLinkBody,
    responses(
        (status = 201, description = "Created", body = LinkDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 409, description = "Link id already exists", body = ErrorBody),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn create_link(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<CreateLinkBody>,
) -> impl IntoResponse {
    let id = body.id.trim().to_string();
    if let Some(r) = validate_id(&id) {
        return r;
    }
    if let Some(r) = validate_link(&body.link) {
        return r;
    }
    if let Some(r) = validate_name(&body.name) {
        return r;
    }

    let model = links::ActiveModel {
        id: Set(id),
        enabled: Set(body.enabled),
        link: Set(body.link),
        prefix_zeros: Set(body.prefix_zeros),
        name: Set(body.name),
    };

    match model.insert(&state.db).await {
        Ok(row) => (StatusCode::CREATED, Json(LinkDto::from(row))).into_response(),
        Err(sea_orm::DbErr::Exec(sea_orm::RuntimeErr::SqlxError(sqlx_err)))
            if sqlx_err
                .as_database_error()
                .is_some_and(|e| e.is_unique_violation()) =>
        {
            json_error(StatusCode::CONFLICT, "link id already exists")
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// `PATCH /api/links/{id}` — update a short link.
#[utoipa::path(
    patch,
    path = "/{id}",
    operation_id = "links.update",
    tag = "links",
    security(("access_cookie" = [])),
    params(("id" = String, Path, description = "Short link id")),
    request_body = UpdateLinkBody,
    responses(
        (status = 200, description = "Updated", body = LinkDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn update_link(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateLinkBody>,
) -> impl IntoResponse {
    if let Some(r) = validate_link(&body.link) {
        return r;
    }
    if let Some(r) = validate_name(&body.name) {
        return r;
    }

    let row = match links::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(row)) => row,
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "link not found"),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let mut model: links::ActiveModel = row.into();
    model.enabled = Set(body.enabled);
    model.link = Set(body.link);
    model.prefix_zeros = Set(body.prefix_zeros);
    model.name = Set(body.name);

    match model.update(&state.db).await {
        Ok(row) => Json(LinkDto::from(row)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// `DELETE /api/links/{id}` — delete a short link.
#[utoipa::path(
    delete,
    path = "/{id}",
    operation_id = "links.delete",
    tag = "links",
    security(("access_cookie" = [])),
    params(("id" = String, Path, description = "Short link id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn delete_link(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = match links::Entity::delete_by_id(id).exec(&state.db).await {
        Ok(r) => r,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if result.rows_affected == 0 {
        return json_error(StatusCode::NOT_FOUND, "link not found");
    }

    StatusCode::NO_CONTENT.into_response()
}
