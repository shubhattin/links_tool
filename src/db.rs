use crate::entities::links;
use crate::redirect::Link;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait};
use std::env;

pub type DbPool = DatabaseConnection;

#[derive(Debug)]
pub enum LookupError {
    Db(DbErr),
}

impl std::fmt::Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LookupError::Db(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LookupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LookupError::Db(e) => Some(e),
        }
    }
}

/// Connect to Postgres (Rustls-backed when using `postgresql://…` URLs, e.g. Neon).
pub async fn connect(database_url: &str) -> Result<DbPool, DbErr> {
    match env::var("DATABASE_SCHEMA")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        Some(schema) => {
            let connect_options = ConnectOptions::new(database_url)
                .set_schema_search_path(schema)
                .to_owned();
            Database::connect(connect_options).await
        }
        None => Database::connect(database_url).await,
    }
}

pub async fn lookup_link(db: &DbPool, id: &str) -> Result<Option<Link>, LookupError> {
    links::Entity::find_by_id(id.to_owned())
        .one(db)
        .await
        .map_err(LookupError::Db)
}
