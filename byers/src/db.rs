use num_traits::cast::ToPrimitive;
use sqlx::types::BigDecimal;

// generate a macro that accepts an sqlx PgPool and a block of code and runs it and at the end, runs self.update(db)
#[macro_export]
macro_rules! update {
    ($db:expr, $self:ident, $code:block) => {{
        $code;
        $self.update($db).await?;
    }};
}

pub struct DbUser {
    pub id: i64,
    pub youtube_channel_id: Option<String>,
    pub watched_time: BigDecimal,
    pub boonbucks: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub last_message_sent: Option<chrono::NaiveDateTime>,
    pub migrated: bool,
}

impl Default for DbUser {
    fn default() -> Self {
        Self {
            id: 0,
            youtube_channel_id: None,
            watched_time: BigDecimal::from(0),
            boonbucks: 0,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            last_message_sent: None,
            migrated: false,
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
            RETURNING id, youtube_channel_id, watched_time, boonbucks, created_at, updated_at, last_message_sent, migrated
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
            SET youtube_channel_id = $1, watched_time = $2, boonbucks = $3, last_message_sent = $4, migrated = $5, updated_at = now()
            WHERE id = $6
            "#,
            self.youtube_channel_id,
            self.watched_time,
            self.boonbucks,
            self.last_message_sent,
            self.migrated,
            self.id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, db: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, youtube_channel_id, watched_time, boonbucks, last_message_sent, migrated)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id)
            DO UPDATE SET youtube_channel_id = $2, watched_time = $3, boonbucks = $4, last_message_sent = $5, migrated = $6, updated_at = now()
            "#,
            self.id,
            self.youtube_channel_id,
            self.watched_time,
            self.boonbucks,
            self.last_message_sent,
            self.migrated
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

        let rank = sqlx::query_as!(
            DbSlcbRank,
            r#"
            SELECT * FROM slcb_rank
            WHERE hour_requirement <= $1
            AND (channel_id IS NULL OR channel_id = $2)
            ORDER BY hour_requirement DESC
            LIMIT 1
            "#,
            user_hours_floor,
            user.youtube_channel_id
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

        let rank = sqlx::query_as!(
            DbSlcbRank,
            r#"
            SELECT * FROM slcb_rank
            WHERE hour_requirement > $1
            AND (channel_id IS NULL OR channel_id = $2)
            ORDER BY hour_requirement ASC
            LIMIT 1
            "#,
            user_hours_floor,
            user.youtube_channel_id
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
}

#[derive(Debug, Clone)]
pub struct DbServerChannelConfig {
    pub id: i64,
    pub server_id: String,
    pub allow_watch_time_accumulation: bool,
    pub allow_point_accumulation: bool,
}

#[derive(Debug, Clone)]
pub struct DbServerConfig {
    pub id: i64,
    pub slot_jackpot: i32,
}

impl DbServerConfig {
    pub async fn fetch_or_insert(db: &sqlx::PgPool, id: i64) -> Result<Self, sqlx::Error> {
        let config = sqlx::query_as!(
            DbServerConfig,
            r#"
            INSERT INTO server_config (id)
            VALUES ($1)
            ON CONFLICT (id)
            DO NOTHING
            RETURNING *
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
            SET slot_jackpot = $2
            WHERE id = $1
            "#,
            self.id,
            self.slot_jackpot
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
