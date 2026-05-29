use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Links {
    Table,
    Id,
    Enabled,
    Link,
    PrefixZeros,
    Name,
}

#[derive(DeriveIden)]
enum Others {
    Table,
    Key,
    Value,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Links::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Links::Id)
                            .string_len(20)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Links::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Links::Link).text().not_null())
                    .col(
                        ColumnDef::new(Links::PrefixZeros)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Links::Name).string_len(30).null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Others::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Others::Key)
                            .string_len(20)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Others::Value).text().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Others::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Links::Table).to_owned())
            .await?;
        Ok(())
    }
}
