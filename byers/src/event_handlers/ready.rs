use std::sync::Arc;

use clokwerk::AsyncScheduler;
use fred::{prelude::PubsubInterface, types::RedisValue};
use poise::serenity_prelude::*;
use tracing::{debug, info};
use tracing_unwrap::ResultExt;
use clokwerk::TimeUnits;

use crate::prelude::*;
use judeharley::{
    communication::ByersUnixStream,
    prelude::{ServerChannelConfig, Songs}, sea_orm::{ActiveValue, DatabaseConnection},
};

async fn spawn_subscriber_handler(
    data: &Data<ByersUnixStream>,
    ctx: &poise::serenity_prelude::Context,
) -> Result<(), crate::prelude::Error> {
    info!("Spawning Redis subscriber message handler...");
    let mut message_rx = data.redis_subscriber.on_message();
    let context = ctx.clone();
    tokio::spawn(async move {
        while let Ok(message) = message_rx.recv().await {
            debug!(
                "Received message {:?} on channel {:?}",
                message.value, message.channel
            );

            match message.channel.to_string().as_str() {
                "byers:status" => {
                    if let RedisValue::String(song) = message.value {
                        context.set_activity(Some(ActivityData::listening(song.to_string())));
                    }
                }
                "moo" => {}
                _ => {}
            }
        }
    });

    Ok(())
}

async fn remind_users_to_hydrate(http: Arc<Http>, db: DatabaseConnection) {
    info!("Sending hydration reminder");

    let hydration_channels = ServerChannelConfig::get_all_hydration_channels(&db)
        .await
        .expect_or_log("Failed to fetch hydration channels");

    for channel in hydration_channels {
        if let Some(last_message_sent) = channel.last_message_sent.as_ref() {
            // if the last message was sent more than 15 minutes ago, don't remind
            if last_message_sent < &(chrono::Utc::now().naive_utc() - chrono::Duration::minutes(15)) {
                continue;
            }
        }

        channel.update(judeharley::entities::server_channel_config::ActiveModel {
            id: ActiveValue::set(channel.id),
            last_message_sent: ActiveValue::set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        }, &db).await.expect_or_log("Failed to update hydration channel");

        let discord_channel_id = ChannelId::new(channel.id as u64);

        discord_channel_id
            .send_message(
                &http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("Hydration reminder")
                        .description("Remember to drink some water 🥤!"),
                ),
            )
            .await
            .expect_or_log("Failed to send hydration reminder");
    }
}

pub async fn on_ready(
    ctx: &poise::serenity_prelude::Context,
    data_about_bot: &poise::serenity_prelude::Ready,
    data: &Data<ByersUnixStream>,
) -> Result<(), crate::prelude::Error> {
    info!("Connected as {}", data_about_bot.user.name);

    spawn_subscriber_handler(data, ctx).await?;

    let mut scheduler = AsyncScheduler::new();

    let http_clone = ctx.http.clone();
    let db_clone = data.db.clone();
    scheduler.every(15.minutes())
        .run(move || remind_users_to_hydrate(http_clone.to_owned(), db_clone.to_owned()));

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    let current_song = Songs::last_played(&data.db).await;
    if let Ok(Some(current_song)) = current_song {
        ctx.set_activity(Some(ActivityData::listening(format!(
            "{} - {}",
            current_song.album, current_song.title
        ))));
    }

    Ok(())
}
