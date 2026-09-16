use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"create table if not exists songs_fulltext
            (
                song_id varchar(64) not null primary key,
                title varchar(255) not null,
                artist varchar(255) not null,
                album varchar(255) not null,
                tsvector TSVECTOR GENERATED ALWAYS AS (
                    to_tsvector('english', title) ||
                    to_tsvector('english', artist) ||
                    to_tsvector('english', album)
                ) STORED
            );"#,
        )
        .await?;

        // transfer over fulltext index
        db.execute_unprepared(
            r#"insert into songs_fulltext (song_id, title, artist, album)
            select file_hash, title, artist, album from songs;"#,
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Songs::Table)
                    .drop_column(Songs::Tsvector)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"alter table songs add column tsvector TSVECTOR GENERATED ALWAYS AS (
                    to_tsvector('english', title) ||
                    to_tsvector('english', artist) ||
                    to_tsvector('english', album)
                ) STORED"#,
        )
        .await?;

        manager
            .drop_table(Table::drop().table(SongsFulltext::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Songs {
    Table,
    Tsvector,
}

#[derive(DeriveIden)]
enum SongsFulltext {
    Table,
}
