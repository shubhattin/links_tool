use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Account {
    Table,
    ProviderId,
    AccountId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("uidx_account_provider_account_id")
                    .table(Account::Table)
                    .col(Account::ProviderId)
                    .col(Account::AccountId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uidx_account_provider_account_id")
                    .table(Account::Table)
                    .to_owned(),
            )
            .await
    }
}
