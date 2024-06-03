use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

fn grist_col<T: IntoIden>(name: T) -> ColumnDef {
    ColumnDef::new(name)
        .integer()
        .not_null()
        .default(0)
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::WatchedTime)
                            .decimal()
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(Users::Boonbucks)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Users::LastMessageSent).timestamp())
                    .col(
                        ColumnDef::new(Users::Migrated)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(&mut grist_col(Users::Amber))
                    .col(&mut grist_col(Users::Amethyst))
                    .col(&mut grist_col(Users::Artifact))
                    .col(&mut grist_col(Users::Caulk))
                    .col(&mut grist_col(Users::Chalk))
                    .col(&mut grist_col(Users::Cobalt))
                    .col(&mut grist_col(Users::Diamond))
                    .col(&mut grist_col(Users::Garnet))
                    .col(&mut grist_col(Users::Gold))
                    .col(&mut grist_col(Users::Iodine))
                    .col(&mut grist_col(Users::Marble))
                    .col(&mut grist_col(Users::Mercury))
                    .col(&mut grist_col(Users::Quartz))
                    .col(&mut grist_col(Users::Ruby))
                    .col(&mut grist_col(Users::Rust))
                    .col(&mut grist_col(Users::Shale))
                    .col(&mut grist_col(Users::Sulfur))
                    .col(&mut grist_col(Users::Tar))
                    .col(&mut grist_col(Users::Uranium))
                    .col(&mut grist_col(Users::Zillium))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Cans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Cans::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Cans::AddedBy).big_integer().not_null())
                    .col(
                        ColumnDef::new(Cans::AddedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Cans::Legit)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cans-added_by")
                            .from(Cans::Table, Cans::AddedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ConnectedYoutubeAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ConnectedYoutubeAccounts::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ConnectedYoutubeAccounts::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConnectedYoutubeAccounts::YoutubeChannelId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConnectedYoutubeAccounts::YoutubeChannelName)
                            .text()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-connected_youtube_accounts-user_id")
                            .from(
                                ConnectedYoutubeAccounts::Table,
                                ConnectedYoutubeAccounts::UserId,
                            )
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ServerConfig::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServerConfig::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::SlotJackpot)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::DiceRoll)
                            .integer()
                            .not_null()
                            .default(111),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ServerChannelConfig::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServerChannelConfig::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ServerChannelConfig::ServerId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ServerChannelConfig::AllowWatchTimeAccumulation)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ServerChannelConfig::AllowPointAccumulation)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ServerChannelConfig::HydrationReminder)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ServerRoleConfig::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServerRoleConfig::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ServerRoleConfig::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ServerRoleConfig::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ServerRoleConfig::MinimumHours)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SlcbCurrency::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SlcbCurrency::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SlcbCurrency::Username).text().not_null())
                    .col(
                        ColumnDef::new(SlcbCurrency::Points)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SlcbCurrency::Hours)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(SlcbCurrency::UserId).text())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SlcbRank::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SlcbRank::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SlcbRank::RankName).text().not_null())
                    .col(
                        ColumnDef::new(SlcbRank::HourRequirement)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SlcbRank::ChannelId).text())
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"create table if not exists songs
            (
                file_path varchar(255) not null
                    primary key,
                title varchar(255) not null,
                artist varchar(255) not null,
                album varchar(255) not null,
                played integer default 0 not null,
                requested integer default 0 not null,
                tsvector TSVECTOR GENERATED ALWAYS AS (
                    to_tsvector('english', title) ||
                    to_tsvector('english', artist) ||
                    to_tsvector('english', album)
                ) STORED,
                duration double precision default 0 not null,
                file_hash varchar(64) not null unique,
                bitrate integer default 0 not null
            );"#,
        )
        .await?;

        manager
            .create_table(
                Table::create()
                    .table(FavouriteSongs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FavouriteSongs::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FavouriteSongs::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FavouriteSongs::SongId)
                            .string_len(255)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("favourite_songs_user_id")
                            .from(FavouriteSongs::Table, FavouriteSongs::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PlayedSongs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlayedSongs::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PlayedSongs::SongId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PlayedSongs::PlayedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SongRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SongRequests::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SongRequests::SongId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SongRequests::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SongRequests::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SongTags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SongTags::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SongTags::SongId).string_len(255).not_null())
                    .col(ColumnDef::new(SongTags::Tag).text().not_null())
                    .col(ColumnDef::new(SongTags::Value).text().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SongTags::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SongRequests::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(PlayedSongs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(FavouriteSongs::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(ConnectedYoutubeAccounts::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Cans::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ServerChannelConfig::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ServerConfig::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ServerRoleConfig::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SlcbCurrency::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SlcbRank::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE songs;").await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Cans {
    Table,
    Id,
    AddedBy,
    AddedAt,
    Legit,
}

#[derive(DeriveIden)]
enum ConnectedYoutubeAccounts {
    Table,
    Id,
    UserId,
    YoutubeChannelId,
    YoutubeChannelName,
}

#[derive(DeriveIden)]
enum FavouriteSongs {
    Table,
    Id,
    UserId,
    SongId,
}

#[derive(DeriveIden)]
enum PlayedSongs {
    Table,
    Id,
    SongId,
    PlayedAt,
}

#[derive(DeriveIden)]
enum ServerChannelConfig {
    Table,
    Id,
    ServerId,
    AllowWatchTimeAccumulation,
    AllowPointAccumulation,
    HydrationReminder,
}

#[derive(DeriveIden)]
enum ServerConfig {
    Table,
    Id,
    SlotJackpot,
    DiceRoll,
}

#[derive(DeriveIden)]
enum ServerRoleConfig {
    Table,
    Id,
    GuildId,
    RoleId,
    MinimumHours,
}

#[derive(DeriveIden)]
enum SlcbCurrency {
    Table,
    Id,
    Username,
    Points,
    Hours,
    UserId,
}

#[derive(DeriveIden)]
enum SlcbRank {
    Table,
    Id,
    RankName,
    HourRequirement,
    ChannelId,
}

#[derive(DeriveIden)]
enum SongRequests {
    Table,
    Id,
    SongId,
    UserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum SongTags {
    Table,
    Id,
    SongId,
    Tag,
    Value,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    WatchedTime,
    Boonbucks,
    CreatedAt,
    UpdatedAt,
    LastMessageSent,
    Migrated,
    Amber,
    Amethyst,
    Artifact,
    Caulk,
    Chalk,
    Cobalt,
    Diamond,
    Garnet,
    Gold,
    Iodine,
    Marble,
    Mercury,
    Quartz,
    Ruby,
    Rust,
    Shale,
    Sulfur,
    Tar,
    Uranium,
    Zillium,
}
