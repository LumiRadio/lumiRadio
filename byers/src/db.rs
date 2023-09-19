use std::fmt::Display;

use anyhow::{anyhow, bail};
use chrono::NaiveDateTime;
use num_traits::cast::ToPrimitive;
use sqlx::{types::BigDecimal, PgPool};

use crate::discord::DiscordConnection;

// generate a macro that accepts an sqlx PgPool and a block of code and runs it and at the end, runs self.update(db)
#[macro_export]
macro_rules! update {
    ($db:expr, $self:ident, $code:block) => {{
        $code
        $self.update($db).await?;
    }};
}

// SELECT title, artist, album, file_path, duration
// FROM songs
// WHERE file_path = $1
pub struct DbSong {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub file_path: String,
    pub duration: f64,
    pub file_hash: String,
}

impl Display for DbSong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{} - {}", self.artist, self.title))
    }
}

impl DbSong {
    pub async fn last_played_song(db: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            DbSong,
            r#"
            SELECT songs.title, songs.artist, songs.album, songs.file_path, songs.duration, songs.file_hash
            FROM songs, played_songs
            WHERE songs.file_path = played_songs.song_id
            ORDER BY played_songs.played_at DESC
            LIMIT 1
            "#,
        )
        .fetch_one(db)
        .await
    }

    pub async fn fetch(db: &sqlx::PgPool, file_path: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbSong,
            r#"
            SELECT title, artist, album, file_path, duration, file_hash
            FROM songs
            WHERE file_path = $1
            "#,
            file_path
        )
        .fetch_optional(db)
        .await
    }

    pub async fn fetch_from_hash(
        db: &sqlx::PgPool,
        file_hash: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbSong,
            r#"
            SELECT title, artist, album, file_path, duration, file_hash
            FROM songs
            WHERE file_hash = $1
            "#,
            file_hash
        )
        .fetch_optional(db)
        .await
    }

    pub async fn search(db: &sqlx::PgPool, query: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbSong,
            r#"
            WITH search AS (
                SELECT to_tsquery(string_agg(lexeme || ':*', ' & ' ORDER BY positions)) AS query
                FROM unnest(to_tsvector($1))
            )
            SELECT title, artist, album, file_path, duration, file_hash
            FROM songs, search
            WHERE tsvector @@ query
            "#,
            query
        )
        .fetch_all(db)
        .await
    }

    pub async fn last_requested(&self, db: &sqlx::PgPool) -> Result<NaiveDateTime, sqlx::Error> {
        let last_played = sqlx::query!(
            r#"
            SELECT created_at
            FROM song_requests
            WHERE song_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            self.file_path
        )
        .fetch_optional(db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query database: {}", e);
            e
        })?;

        let last_played = if let Some(last_played) = last_played {
            last_played.created_at
        } else {
            chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
        };

        Ok(last_played)
    }

    pub async fn request(&self, db: &sqlx::PgPool, author_id: u64) -> Result<(), sqlx::Error> {
        DbUser::fetch_or_insert(db, author_id as i64).await?;

        sqlx::query!(
            r#"
            INSERT INTO song_requests (song_id, user_id)
            VALUES ($1, $2)
            "#,
            self.file_path,
            author_id as i64
        )
        .execute(db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert song request: {}", e);
            e
        })?;

        Ok(())
    }
}

pub struct DbUser {
    pub id: i64,
    pub watched_time: BigDecimal,
    pub boonbucks: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub last_message_sent: Option<chrono::NaiveDateTime>,
    pub migrated: bool,
    pub amber: i32,
    pub amethyst: i32,
    pub artifact: i32,
    pub caulk: i32,
    pub chalk: i32,
    pub cobalt: i32,
    pub diamond: i32,
    pub garnet: i32,
    pub gold: i32,
    pub iodine: i32,
    pub marble: i32,
    pub mercury: i32,
    pub quartz: i32,
    pub ruby: i32,
    pub rust: i32,
    pub shale: i32,
    pub sulfur: i32,
    pub tar: i32,
    pub uranium: i32,
    pub zillium: i32,
}

impl Default for DbUser {
    fn default() -> Self {
        Self {
            id: 0,
            watched_time: BigDecimal::from(0),
            boonbucks: 0,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            last_message_sent: None,
            migrated: false,
            amber: 0,
            amethyst: 0,
            artifact: 0,
            caulk: 0,
            chalk: 0,
            cobalt: 0,
            diamond: 0,
            garnet: 0,
            gold: 0,
            iodine: 0,
            marble: 0,
            mercury: 0,
            quartz: 0,
            ruby: 0,
            rust: 0,
            shale: 0,
            sulfur: 0,
            tar: 0,
            uranium: 0,
            zillium: 0,
        }
    }
}

impl DbUser {
    pub async fn fetch(db: &sqlx::PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbUser,
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn fetch_or_insert(db: &sqlx::PgPool, id: i64) -> Result<Self, sqlx::Error> {
        if let Some(user) = Self::fetch(db, id).await? {
            return Ok(user);
        }

        sqlx::query_as!(
            DbUser,
            r#"
            INSERT INTO users (id) VALUES ($1)
            ON CONFLICT (id) DO NOTHING
            RETURNING id, watched_time, boonbucks, created_at, updated_at, last_message_sent, migrated, amber, amethyst, artifact, caulk, chalk, cobalt, diamond, garnet, gold, iodine, marble, mercury, quartz, ruby, rust, shale, sulfur, tar, uranium, zillium
            "#,
            id
        )
        .fetch_one(db)
        .await
    }

    pub async fn update(&self, db: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE users
            SET watched_time = $2, boonbucks = $3, created_at = $4, updated_at = $5, last_message_sent = $6, migrated = $7, amber = $8, amethyst = $9, artifact = $10, caulk = $11, chalk = $12, cobalt = $13, diamond = $14, garnet = $15, gold = $16, iodine = $17, marble = $18, mercury = $19, quartz = $20, ruby = $21, rust = $22, shale = $23, sulfur = $24, tar = $25, uranium = $26, zillium = $27
            WHERE id = $1
            "#,
            self.id,
            self.watched_time,
            self.boonbucks,
            self.created_at,
            self.updated_at,
            self.last_message_sent,
            self.migrated,
            self.amber,
            self.amethyst,
            self.artifact,
            self.caulk,
            self.chalk,
            self.cobalt,
            self.diamond,
            self.garnet,
            self.gold,
            self.iodine,
            self.marble,
            self.mercury,
            self.quartz,
            self.ruby,
            self.rust,
            self.shale,
            self.sulfur,
            self.tar,
            self.uranium,
            self.zillium
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn fetch_position_in_hours(&self, db: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
        let position = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM users
            WHERE watched_time > $1
            "#,
            self.watched_time
        )
        .fetch_one(db)
        .await?;

        let position = position.count.unwrap();
        // add 1 because we want the position, not the rank
        Ok(position + 1)
    }

    pub async fn fetch_position_in_boonbucks(&self, db: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
        let position = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM users
            WHERE boonbucks > $1
            "#,
            self.boonbucks
        )
        .fetch_one(db)
        .await?;

        let position = position.count.unwrap();
        // add 1 because we want the position, not the rank
        Ok(position + 1)
    }

    pub async fn add_linked_channels(
        &self,
        db: &sqlx::PgPool,
        channels: Vec<DiscordConnection>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM connected_youtube_accounts
            WHERE user_id = $1
            "#,
            self.id
        )
        .execute(db)
        .await?;

        for channel in channels {
            sqlx::query!(r#"
            INSERT INTO connected_youtube_accounts (user_id, youtube_channel_id, youtube_channel_name)
            VALUES ($1, $2, $3)
            "#, self.id, channel.id, channel.name)
                .execute(db)
                .await?;
        }

        Ok(())
    }

    pub async fn linked_channels(
        &self,
        db: &sqlx::PgPool,
    ) -> Result<Vec<DbConnectedAccount>, sqlx::Error> {
        sqlx::query_as!(
            DbConnectedAccount,
            r#"
            SELECT * FROM connected_youtube_accounts
            WHERE user_id = $1
            "#,
            self.id
        )
        .fetch_all(db)
        .await
    }
}

#[derive(Debug, Clone)]
pub struct DbConnectedAccount {
    pub id: i32,
    pub user_id: i64,
    pub youtube_channel_id: String,
    pub youtube_channel_name: String,
}

#[derive(Debug, Clone)]
pub struct DbSlcbRank {
    pub id: i32,
    pub rank_name: String,
    pub hour_requirement: i32,
    pub channel_id: Option<String>,
}

impl DbSlcbRank {
    pub async fn fetch_rank_for_user(
        user: &DbUser,
        db: &sqlx::PgPool,
    ) -> Result<String, sqlx::Error> {
        // fetch the rank for the user based on the hour requirement
        // additionally, if the rank has a channel_id, check if the user has a channel_id
        // if the user has a channel_id, check both the hour requirement and the channel_id
        let user_hours_floor: i32 = user.watched_time.round(0).to_i32().unwrap();
        let linked_channels = user
            .linked_channels(db)
            .await?
            .into_iter()
            .map(|c| c.youtube_channel_id)
            .collect::<Vec<_>>();

        let rank = sqlx::query_as!(
            DbSlcbRank,
            r#"
            SELECT * FROM slcb_rank
            WHERE hour_requirement <= $1
            AND (channel_id IS NULL OR channel_id = ANY($2))
            ORDER BY hour_requirement DESC
            LIMIT 1
            "#,
            user_hours_floor,
            &linked_channels[..]
        )
        .fetch_optional(db)
        .await?;

        Ok(rank
            .map(|r| r.rank_name)
            .unwrap_or("Wow, literally no rank available...".to_string()))
    }

    pub async fn fetch_next_rank_for_user(
        user: &DbUser,
        db: &sqlx::PgPool,
    ) -> Result<Option<DbSlcbRank>, sqlx::Error> {
        // fetch the rank for the user based on the hour requirement
        // additionally, if the rank has a channel_id, check if the user has a channel_id
        // if the user has a channel_id, check both the hour requirement and the channel_id
        let user_hours_floor: i32 = user.watched_time.round(0).to_i32().unwrap();
        let linked_channels = user
            .linked_channels(db)
            .await?
            .into_iter()
            .map(|c| c.youtube_channel_id)
            .collect::<Vec<_>>();

        let rank = sqlx::query_as!(
            DbSlcbRank,
            r#"
            SELECT * FROM slcb_rank
            WHERE hour_requirement > $1
            AND (channel_id IS NULL OR channel_id = ANY($2))
            ORDER BY hour_requirement ASC
            LIMIT 1
            "#,
            user_hours_floor,
            &linked_channels[..]
        )
        .fetch_optional(db)
        .await?;

        Ok(rank)
    }
}

#[derive(Debug, Clone)]
pub struct DbSlcbUser {
    pub id: i32,
    pub username: String,
    pub points: i32,
    pub hours: i32,
    pub user_id: Option<String>,
}

impl DbSlcbUser {
    pub async fn fetch(db: &sqlx::PgPool, id: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbSlcbUser,
            r#"
            SELECT * FROM slcb_currency
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(db)
        .await
    }
    
    pub async fn fetch_by_user_id(db: &PgPool, user_id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbSlcbUser,
            r#"
            SELECT * FROM slcb_currency
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn fetch_by_username(
        db: &sqlx::PgPool,
        username: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbSlcbUser,
            r#"
            SELECT * FROM slcb_currency
            WHERE username = $1
            "#,
            username
        )
        .fetch_all(db)
        .await
    }

    pub async fn search_by_username(
        db: &sqlx::PgPool,
        username: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbSlcbUser,
            r#"
            SELECT * FROM slcb_currency
            WHERE username ILIKE $1
            "#,
            format!("%{}%", username)
        )
        .fetch_all(db)
        .await
    }
}

#[derive(Debug, Clone)]
pub struct DbServerConfig {
    pub id: i64,
    pub slot_jackpot: i32,
    pub dice_roll: i32,
}

impl DbServerConfig {
    pub async fn fetch_or_insert(db: &sqlx::PgPool, id: i64) -> Result<Self, sqlx::Error> {
        let config = sqlx::query_as!(
            DbServerConfig,
            r#"
            SELECT * FROM server_config
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(db)
        .await?;

        if let Some(config) = config {
            return Ok(config);
        }

        let config = sqlx::query_as!(
            DbServerConfig,
            r#"
            INSERT INTO server_config (id)
            VALUES ($1)
            ON CONFLICT (id)
            DO NOTHING
            RETURNING id, slot_jackpot, dice_roll
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(config)
    }

    pub async fn update(&self, db: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE server_config
            SET slot_jackpot = $2, dice_roll = $3
            WHERE id = $1
            "#,
            self.id,
            self.slot_jackpot,
            self.dice_roll
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DbServerChannelConfig {
    pub id: i64,
    pub server_id: i64,
    pub allow_watch_time_accumulation: bool,
    pub allow_point_accumulation: bool,
}

impl DbServerChannelConfig {
    pub async fn fetch(
        db: &PgPool,
        channel_id: i64,
        server_id: i64,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            DbServerChannelConfig,
            r#"
            SELECT * FROM server_channel_config
            WHERE id = $1 AND server_id = $2
            "#,
            channel_id,
            server_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn fetch_or_insert(
        db: &PgPool,
        channel_id: i64,
        server_id: i64,
    ) -> Result<Self, sqlx::Error> {
        if let Some(config) = Self::fetch(db, channel_id, server_id).await? {
            return Ok(config);
        }

        let config = sqlx::query_as!(
            DbServerChannelConfig,
            r#"
            INSERT INTO server_channel_config (id, server_id)
            VALUES ($1, $2)
            ON CONFLICT (id)
            DO NOTHING
            RETURNING id, server_id, allow_watch_time_accumulation, allow_point_accumulation
            "#,
            channel_id,
            server_id
        )
        .fetch_one(db)
        .await?;

        Ok(config)
    }

    pub async fn update(&self, db: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE server_channel_config
            SET allow_watch_time_accumulation = $2, allow_point_accumulation = $3
            WHERE id = $1
            "#,
            self.id,
            self.allow_watch_time_accumulation,
            self.allow_point_accumulation
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DbCan {
    pub id: i32,
    pub added_by: i64,
    pub added_at: chrono::NaiveDateTime,
    pub legit: bool,
}

impl DbCan {
    pub async fn add_one(db: &PgPool, added_by: i64, legit: bool) -> Result<(), sqlx::Error> {
        DbUser::fetch_or_insert(db, added_by).await?;

        sqlx::query!(
            r#"
            INSERT INTO cans (added_by, legit)
            VALUES ($1, $2)
            "#,
            added_by,
            legit
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn count(db: &PgPool) -> Result<i64, sqlx::Error> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM cans
            "#,
        )
        .fetch_one(db)
        .await?;

        Ok(count.count.unwrap())
    }

    pub async fn count_for_user(db: &PgPool, user_id: i64) -> Result<i64, sqlx::Error> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM cans
            WHERE added_by = $1 AND legit = true
            "#,
            user_id
        )
        .fetch_one(db)
        .await?;

        Ok(count.count.unwrap())
    }

    pub async fn add_multiple(db: &PgPool, added_by: i64, amount: i32) -> Result<(), sqlx::Error> {
        if amount <= 0 {
            return Ok(());
        }

        // insert amount rows into the database
        // with the same added_by and legit
        let mut query = String::new();
        query.push_str("INSERT INTO cans (added_by, legit) VALUES ");
        for _ in 0..amount {
            query.push_str("($1, $2),");
        }
        query.pop(); // remove the last comma
        query.push(';');
        sqlx::query(&query)
            .bind(added_by)
            .bind(false)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn set(db: &PgPool, added_by: i64, amount: i32) -> Result<(), anyhow::Error> {
        if amount <= 0 {
            return Ok(());
        }

        let current_count = DbCan::count(db).await?;
        if amount <= current_count as i32 {
            DbCan::remove_last_n(db, current_count as i32 - amount).await?;
            return Ok(());
        }

        let to_be_added = amount - current_count as i32;
        DbCan::add_multiple(db, added_by, to_be_added).await?;

        Ok(())
    }

    pub async fn remove_last_n(db: &PgPool, amount: i32) -> Result<(), anyhow::Error> {
        if amount <= 0 {
            return Ok(());
        }

        let current_count = DbCan::count(db).await?;
        if amount > current_count as i32 {
            bail!("You can't remove more cans than there are in the database");
        }

        sqlx::query!(
            r#"
            DELETE FROM cans
            WHERE id IN (
                SELECT id FROM cans
                ORDER BY id DESC
                LIMIT $1
            )
            "#,
            amount as i64
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
