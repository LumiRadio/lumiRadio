use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("UPDATE users u SET watched_time = ou.watched_time * 3600 FROM users ou WHERE u.id = ou.id;").await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(
                        ColumnDef::new(Users::WatchedTime)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(
                        ColumnDef::new(Users::WatchedTime)
                            .decimal()
                            .not_null()
                            .default(0.0),
                    )
                    .to_owned(),
            )
            .await?;
        
        let db = manager.get_connection();

        db.execute_unprepared("UPDATE users u SET watched_time = ou.watched_time / 3600 FROM users ou WHERE u.id = ou.id;").await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    WatchedTime,
}
