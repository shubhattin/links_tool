#![allow(dead_code)]
//! Optional standalone [`PgPool`] for raw queries when SeaORM DSL is insufficient.
//! SeaORM owns the primary Postgres connection (`DatabaseConnection`). Use this sparingly.

pub type PgPool = sqlx::PgPool;

/// Build a pooled connection using the same Postgres URL SeaORM consumes.
pub async fn sql_fallback_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
