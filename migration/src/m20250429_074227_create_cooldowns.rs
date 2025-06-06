use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Cooldown::Table)
                    .col(pk_auto(Cooldown::Id))
                    .col(string(Cooldown::Scope))
                    .col(big_integer_null(Cooldown::UserId))
                    .col(string(Cooldown::Key))
                    .col(timestamp(Cooldown::ExpiresAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Cooldown::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Cooldown {
    Table,
    Id,
    Scope,
    UserId,
    Key,
    ExpiresAt,
}
