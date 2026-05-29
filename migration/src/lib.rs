pub use sea_orm_migration::prelude::*;

mod m20260419120000_initial_schema;
mod m20260529120000_auth_schema;
mod m20260529130000_account_provider_unique;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260419120000_initial_schema::Migration),
            Box::new(m20260529120000_auth_schema::Migration),
            Box::new(m20260529130000_account_provider_unique::Migration),
        ]
    }
}
