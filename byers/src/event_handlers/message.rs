use chrono::NaiveDateTime;
use fred::{
    prelude::{KeysInterface, RedisClient},
    types::Expiration,
};
use poise::serenity_prelude::{self as serenity, ChannelId, GuildId, Message, UserId};
use sqlx::{types::BigDecimal, PgPool};
use tracing::info;
use tracing_unwrap::ResultExt;

use crate::{
    communication::ByersUnixStream,
    db::{DbServerChannelConfig, DbUser},
    prelude::{Data, Error, Wrappable, W},
};

#[async_trait::async_trait]
trait UserMessageHandlerExt {
    fn redis_message_cooldown_key(&self) -> String;
    fn user_id(&self) -> serenity::UserId;
    async fn update_watched_time(&mut self, db: &PgPool) -> Result<(), Error>;
    async fn update_boondollars(
        &mut self,
        redis_client: &RedisClient,
        db: &PgPool,
    ) -> Result<(), Error>;
}

#[async_trait::async_trait]
impl UserMessageHandlerExt for DbUser {
    fn redis_message_cooldown_key(&self) -> String {
        format!("message_cooldown:{}", self.id)
    }

    fn user_id(&self) -> serenity::UserId {
        serenity::UserId(self.id as u64)
    }

    async fn update_watched_time(&mut self, db: &PgPool) -> Result<(), Error> {
        if self.last_message_sent.is_none() {
            // first message
            info!("User {} sent their first message", self.id);
            self.last_message_sent = Some(chrono::Utc::now().naive_utc());
        } else {
            let now = chrono::Utc::now().naive_utc();
            // check if the user has sent a message in the last 15 minutes
            // if so, add the time difference to their watched time
            // if not, do nothing
            let time_diff = now - self.last_message_sent.unwrap();
            self.last_message_sent = Some(now);

            if time_diff.num_minutes() <= 15 {
                info!(
                    "User {} sent a message within 15 minutes, adding {} seconds to their watched time",
                    self.id,
                    time_diff.num_seconds()
                );

                self.watched_time += BigDecimal::from(time_diff.num_seconds()) / 3600;
            }
        }

        self.update(db).await?;

        Ok(())
    }

    async fn update_boondollars(
        &mut self,
        redis_client: &RedisClient,
        db: &PgPool,
    ) -> Result<(), Error> {
        let cooldown_key = self.redis_message_cooldown_key();
        if let Some(cooldown) = redis_client.get::<Option<String>, _>(&cooldown_key).await? {
            let cooldown = NaiveDateTime::parse_from_str(&cooldown, "%Y-%m-%d %H:%M:%S%.f")?;

            if cooldown > chrono::Utc::now().naive_utc() {
                return Ok(());
            }
        }

        redis_client
            .set(
                &cooldown_key,
                (chrono::Utc::now() + chrono::Duration::minutes(5))
                    .naive_utc()
                    .to_string(),
                Some(Expiration::EX(300)),
                None,
                false,
            )
            .await?;

        info!("User {} sent a message, awarding 3 boonbucks", self.id);

        self.boonbucks += 3;

        self.update(db).await?;

        Ok(())
    }
}

pub async fn update_activity(
    data: &Data<ByersUnixStream>,
    author: UserId,
    channel_id: ChannelId,
    guild_id: GuildId,
) -> Result<(), Error> {
    let Some(channel_config) = DbServerChannelConfig::fetch(&data.db, channel_id.0 as i64, guild_id.0 as i64).await? else {
        return Ok(());
    };

    let mut user = DbUser::fetch_or_insert(&data.db, author.0 as i64).await?;

    if channel_config.allow_watch_time_accumulation {
        user.update_watched_time(&data.db).await?;
    }
    if channel_config.allow_point_accumulation {
        user.update_boondollars(&data.redis_pool, &data.db).await?;
    }

    Ok(())
}

pub async fn message_handler(message: &Message, data: &Data<ByersUnixStream>) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

    let Some(guild_id) = message.guild_id else {
        return Ok(());
    };

    update_activity(data, message.author.id, message.channel_id, guild_id).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_datetime() {
        let now = chrono::Utc::now().naive_utc();
        let now_str = now.to_string();
        println!("{}", now_str);

        // parse 2023-09-19 12:39:33.359969291 as UTC
        let parsed =
            chrono::NaiveDateTime::parse_from_str(&now_str, "%Y-%m-%d %H:%M:%S%.f").unwrap();

        assert_eq!(now, parsed);
    }
}
