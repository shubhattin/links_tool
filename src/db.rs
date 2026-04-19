use crate::redirect::{self, Link};
use diesel_async::pooled_connection::deadpool::{BuildError, Pool, PoolError};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

/// Shared PostgreSQL pool (one per serverless instance / process).
pub type DbPool = Pool<AsyncPgConnection>;

#[derive(Debug)]
pub enum LookupError {
    Pool(PoolError),
    Query(diesel::result::Error),
}

impl std::fmt::Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LookupError::Pool(e) => write!(f, "connection pool: {e}"),
            LookupError::Query(e) => write!(f, "query: {e}"),
        }
    }
}

impl std::error::Error for LookupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LookupError::Pool(e) => Some(e),
            LookupError::Query(e) => Some(e),
        }
    }
}

/// Build a pool from `PG_DATABASE_URL` (call once at startup).
pub fn build_pool(database_url: &str) -> Result<DbPool, BuildError> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(config).build()
}

/// Load a link by short id using an async pooled connection.
pub async fn lookup_link(pool: &DbPool, id: &str) -> Result<Option<Link>, LookupError> {
    let mut conn = pool.get().await.map_err(LookupError::Pool)?;
    redirect::find_link_by_id(&mut conn, id)
        .await
        .map_err(LookupError::Query)
}
