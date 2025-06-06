use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TokenStorage::Table)
                    .if_not_exists()
                    .col(pk_auto(TokenStorage::Id))
                    .col(big_integer_uniq(TokenStorage::UserId))
                    .col(string(TokenStorage::AccessToken))
                    .col(string(TokenStorage::RefreshToken))
                    .col(timestamp(TokenStorage::ExpiresAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TokenStorage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TokenStorage {
    Table,
    Id,
    UserId,
    AccessToken,
    RefreshToken,
    ExpiresAt,
}
