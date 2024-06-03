use chrono::NaiveDateTime;
use fred::{
    prelude::{KeysInterface, RedisClient},
    types::Expiration,
};
use poise::serenity_prelude::{ChannelId, Message, UserId};
use tracing::info;

use crate::prelude::*;
use judeharley::{
    communication::ByersUnixStream,
    prelude::Users,
    sea_orm::{ActiveValue, DatabaseConnection, Set},
    ServerChannelConfig,
};

#[async_trait::async_trait]
trait UserMessageHandlerExt: Sized {
    fn redis_message_cooldown_key(&self) -> String;
    async fn update_watched_time(self, db: &DatabaseConnection) -> Result<Self, Error>;
    async fn update_boondollars(
        self,
        redis_client: &RedisClient,
        db: &DatabaseConnection,
    ) -> Result<Self, Error>;
}

#[async_trait::async_trait]
impl UserMessageHandlerExt for Users {
    fn redis_message_cooldown_key(&self) -> String {
        format!("message_cooldown:{}", self.id)
    }

    async fn update_watched_time(self, db: &DatabaseConnection) -> Result<Self, Error> {
        let user = if self.last_message_sent.is_none() {
            // first message
            info!("User {} sent their first message", self.id);
            let last_message_sent = Some(chrono::Utc::now().naive_utc());
            self.update(
                judeharley::entities::users::ActiveModel {
                    last_message_sent: last_message_sent.map_or(ActiveValue::not_set(), |t| Set(Some(t))),
                    ..Default::default()
                },
                db,
            )
            .await?
        } else {
            let now = chrono::Utc::now().naive_utc();
            let time_diff = now - self.last_message_sent.unwrap();
            let last_message_sent = Some(now);

            let watched_time = if time_diff.num_minutes() <= 15 {
                info!(
                    "User {} sent a message within 15 minutes, adding {} seconds to their watched time",
                    self.id,
                    time_diff.num_seconds()
                );

                self.watched_time + time_diff.num_seconds()
            } else {
                info!("User {} sent a message more than 15 minutes ago, only adding 15 minutes to their watched time", self.id);
                self.watched_time + 15 * 60
            };

            self.update(
                judeharley::entities::users::ActiveModel {
                    last_message_sent: last_message_sent.map_or(ActiveValue::not_set(), |t| Set(Some(t))),
                    watched_time: Set(watched_time),
                    ..Default::default()
                },
                db,
            )
            .await?
        };

        Ok(user)
    }

    async fn update_boondollars(
        self,
        redis_client: &RedisClient,
        db: &DatabaseConnection,
    ) -> Result<Self, Error> {
        let cooldown_key = self.redis_message_cooldown_key();
        if let Some(cooldown) = redis_client.get::<Option<String>, _>(&cooldown_key).await? {
            let cooldown = NaiveDateTime::parse_from_str(&cooldown, "%Y-%m-%d %H:%M:%S%.f")?;

            if cooldown > chrono::Utc::now().naive_utc() {
                return Ok(self);
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

        info!("User {} sent a message, awarding 3 Boondollars", self.id);

        let new_boonbucks = self.boonbucks + 3;
        self.update(
            judeharley::entities::users::ActiveModel {
                boonbucks: Set(new_boonbucks),
                ..Default::default()
            },
            db,
        )
        .await
        .map_err(Into::into)
    }
}

pub async fn update_activity(
    data: &Data<ByersUnixStream>,
    author: UserId,
    channel_id: ChannelId,
) -> Result<(), Error> {
    let Some(channel_config) = ServerChannelConfig::get(channel_id.get(), &data.db).await? else {
        return Ok(());
    };

    let user = Users::get_or_insert(author.get(), &data.db).await?;

    let user = if channel_config.allow_watch_time_accumulation {
        user.update_watched_time(&data.db).await?
    } else {
        user
    };
    if channel_config.allow_point_accumulation {
        user.update_boondollars(&data.redis_pool, &data.db).await?
    } else {
        user
    };

    Ok(())
}

pub async fn message_handler(message: &Message, data: &Data<ByersUnixStream>) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

    update_activity(data, message.author.id, message.channel_id).await?;

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
