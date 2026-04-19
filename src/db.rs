use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

/// PostgreSQL URL from `PG_DATABASE_URL` (matches Drizzle / deployment env naming).
pub fn establish_connection() -> PgConnection {
    let database_url = env::var("PG_DATABASE_URL").expect("PG_DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|e| panic!("Error connecting to PostgreSQL: {e}"))
}
