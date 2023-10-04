use fred::{prelude::PubsubInterface, types::RedisValue};
use poise::serenity_prelude::{Activity, ChannelId};
use tracing::{debug, info};
use tracing_unwrap::ResultExt;

use byers::{
    communication::ByersUnixStream,
    db::{DbServerChannelConfig, DbSong},
    prelude::{Data, Error},
};

async fn spawn_subscriber_handler(
    data: &Data<ByersUnixStream>,
    ctx: &poise::serenity_prelude::Context,
) -> Result<(), Error> {
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
                        context.set_activity(Activity::listening(song)).await;
                    }
                }
                "moo" => {}
                _ => {}
            }
        }
    });

    Ok(())
}

pub async fn on_ready(
    ctx: &poise::serenity_prelude::Context,
    data_about_bot: &poise::serenity_prelude::Ready,
    data: &Data<ByersUnixStream>,
) -> Result<(), Error> {
    info!("Connected as {}", data_about_bot.user.name);

    spawn_subscriber_handler(data, ctx).await?;

    spawn_hydration_reminder(data, ctx).await?;

    let current_song = DbSong::last_played_song(&data.db).await;
    if let Ok(Some(current_song)) = current_song {
        ctx.set_activity(Activity::listening(format!(
            "{} - {}",
            current_song.album, current_song.title
        )))
        .await;
    }

    Ok(())
}

async fn spawn_hydration_reminder(
    data: &Data<ByersUnixStream>,
    ctx: &poise::serenity_prelude::Context,
) -> Result<(), Error> {
    let db = data.db.clone();
    let inner_ctx = ctx.clone();

    tokio::spawn(async move {
        let db = db;
        let ctx = inner_ctx;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15 * 60));
        loop {
            interval.tick().await;

            info!("Sending hydration reminder");

            let hydration_channels = DbServerChannelConfig::fetch_hydration_channels(&db)
                .await
                .expect_or_log("Failed to fetch hydration channels");

            for channel in hydration_channels {
                let discord_channel_id = ChannelId(channel.id as u64);
                discord_channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Hydration reminder")
                                .description("Remember to drink some water 🥤!")
                        })
                    })
                    .await
                    .expect_or_log("Failed to send hydration reminder");
            }
        }
    });

    Ok(())
}
