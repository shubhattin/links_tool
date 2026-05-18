use sea_orm::{ConnectionTrait, DbErr};
use sea_orm_migration::prelude::*;

const UP_LINKS: &str = r#"
CREATE TABLE IF NOT EXISTS links (
    id VARCHAR(20) PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    link TEXT NOT NULL,
    prefix_zeros INTEGER NOT NULL DEFAULT 0,
    name VARCHAR(30)
);
"#;

const UP_OTHERS: &str = r#"
CREATE TABLE IF NOT EXISTS others (
    key VARCHAR(20) PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(UP_LINKS.trim()).await?;
        conn.execute_unprepared(UP_OTHERS.trim()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared("DROP TABLE IF EXISTS others;")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS links;")
            .await?;
        Ok(())
    }
}
