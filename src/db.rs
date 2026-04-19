use crate::redirect::{self, Link};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

/// Shared PostgreSQL pool (one per serverless instance / process).
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Debug)]
pub enum LookupError {
    Pool(r2d2::Error),
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
pub fn build_pool(database_url: &str) -> Result<DbPool, r2d2::Error> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

/// Load a link by short id using a pooled connection (blocking / Diesel sync API).
pub fn lookup_link(pool: &DbPool, id: &str) -> Result<Option<Link>, LookupError> {
    let mut conn = pool.get().map_err(LookupError::Pool)?;
    redirect::find_link_by_id(&mut conn, id).map_err(LookupError::Query)
}
