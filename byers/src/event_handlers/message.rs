use chrono::NaiveDateTime;
use fred::{
    prelude::{KeysInterface, RedisClient},
    types::Expiration,
};
use poise::serenity_prelude::{self as serenity, Message};
use sqlx::{types::BigDecimal, PgPool};
use tracing::info;
use tracing_unwrap::ResultExt;

use crate::{
    db::DbUser,
    prelude::{Data, Error, W},
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
        if let Some(cooldown) = redis_client
            .get::<Option<W<NaiveDateTime>>, _>(&cooldown_key)
            .await?
        {
            if *cooldown > chrono::Utc::now().naive_utc() {
                return Ok(());
            }
        }

        redis_client
            .set(
                &cooldown_key,
                W((chrono::Utc::now() + chrono::Duration::minutes(5)).naive_utc()),
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

pub async fn message_handler(message: &Message, data: &Data) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

    // check if a user exists in the database, if not, add them
    let mut user = DbUser::fetch_or_insert(&data.db, message.author.id.0 as i64).await?;

    user.update_watched_time(&data.db).await?;
    user.update_boondollars(&data.redis_client, &data.db)
        .await?;

    Ok(())
}
