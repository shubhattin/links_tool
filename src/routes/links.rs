//! Links API (mounted at `/api/links` in [`crate::app::router`]).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET` | `/api/links` | [`list_links`] |
//! | `POST` | `/api/links` | [`create_link`] |
//! | `PATCH` | `/api/links/{id}` | [`update_link`] |
//! | `DELETE` | `/api/links/{id}` | [`delete_link`] |

use crate::auth::ErrorBody;
use crate::db::LookupError;
use crate::entities::links;
pub use crate::entities::links::Model as Link;
use crate::state::AppState;
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};

/// One short link and its stored attributes.
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
pub struct LinksListResponse {
    pub links: Vec<LinkDto>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkBody {
    pub id: String,
    pub enabled: bool,
    pub link: String,
    pub prefix_zeros: i32,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
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
    if id.is_empty() || id.chars().count() > 20 {
        return Some(json_error(
            StatusCode::BAD_REQUEST,
            "id must be 1–20 characters",
        ));
    }
    None
}

fn validate_name(name: &Option<String>) -> Option<Response> {
    if let Some(name) = name
        && name.chars().count() > 30
    {
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

/// Links sub-router (nest at `/api/links`).
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_links).post(create_link))
        .route("/{id}", patch(update_link).delete(delete_link))
}

/// `GET /api/links` — all short links (requires access cookie).
pub async fn list_links(State(state): State<AppState>) -> impl IntoResponse {
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
pub async fn create_link(
    State(state): State<AppState>,
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
        Err(err)
            if matches!(
                err.sql_err(),
                Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
            ) || matches!(
                &err,
                sea_orm::DbErr::Exec(sea_orm::RuntimeErr::SqlxError(sqlx_err))
                    if sqlx_err
                        .as_database_error()
                        .is_some_and(|e| e.is_unique_violation())
            ) =>
        {
            json_error(StatusCode::CONFLICT, "link id already exists")
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// `PATCH /api/links/{id}` — update a short link.
pub async fn update_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateLinkBody>,
    // ^ optional: `_user: AuthUser` — FromRequestParts Arc-clones from extensions
    // (insert_auth_info); access fields via Deref (`user.user_id`, `user.role`)
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
pub async fn delete_link(
    State(state): State<AppState>,
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
