use std::sync::Arc;

use clokwerk::AsyncScheduler;
use clokwerk::TimeUnits;
use lazy_static::lazy_static;
use poise::serenity_prelude::*;
use tokio::task::JoinHandle;
use tracing::info;
use tracing_unwrap::ResultExt;

use crate::prelude::*;
use judeharley::{
    communication::ByersUnixStream,
    prelude::{ServerChannelConfig, Songs},
    sea_orm::{ActiveValue, DatabaseConnection},
};

lazy_static! {
    static ref INITIALIZED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
}

async fn set_song(ctx: poise::serenity_prelude::Context, db: DatabaseConnection) {
    let current_song = Songs::last_played(&db).await;
    if let Ok(Some(current_song)) = current_song {
        ctx.set_activity(Some(ActivityData::listening(format!(
            "{} - {}",
            current_song.album, current_song.title
        ))));
    }
}

async fn remind_users_to_hydrate(http: Arc<Http>, db: DatabaseConnection) {
    info!("Sending hydration reminder");

    let hydration_channels = ServerChannelConfig::get_all_hydration_channels(&db)
        .await
        .expect_or_log("Failed to fetch hydration channels");

    for channel in hydration_channels {
        if let Some(last_message_sent) = channel.last_message_sent.as_ref() {
            // if the last message was sent more than 15 minutes ago, don't remind
            if last_message_sent < &(chrono::Utc::now().naive_utc() - chrono::Duration::minutes(15))
            {
                continue;
            }
        }

        channel
            .update(
                judeharley::entities::server_channel_config::ActiveModel {
                    id: ActiveValue::set(channel.id),
                    last_message_sent: ActiveValue::set(Some(chrono::Utc::now().naive_utc())),
                    ..Default::default()
                },
                &db,
            )
            .await
            .expect_or_log("Failed to update hydration channel");

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

fn initialize_scheduler(
    ctx: &poise::serenity_prelude::Context,
    data: &Data<ByersUnixStream>,
) -> Result<AsyncScheduler, crate::prelude::Error> {
    let mut scheduler = AsyncScheduler::new();

    let http_clone = ctx.http.clone();
    let db_clone = data.db.clone();
    scheduler
        .every(15.minutes())
        .run(move || remind_users_to_hydrate(http_clone.to_owned(), db_clone.to_owned()));

    let ctx_clone = ctx.clone();
    let db_clone = data.db.clone();
    scheduler
        .every(10.seconds())
        .run(move || set_song(ctx_clone.to_owned(), db_clone.to_owned()));

    Ok(scheduler)
}

pub async fn on_ready(
    ctx: &poise::serenity_prelude::Context,
    data_about_bot: &poise::serenity_prelude::Ready,
    data: &Data<ByersUnixStream>,
) -> Result<(), crate::prelude::Error> {
    info!("Connected as {}", data_about_bot.user.name);

    if !INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        // spawn_subscriber_handler(data, ctx).await?;

        let mut scheduler = initialize_scheduler(ctx, data)?;

        let handle: JoinHandle<()> = tokio::spawn(async move {
            loop {
                scheduler.run_pending().await;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

        let mut scheduler_handle = data.scheduler_handle.lock().await;
        *scheduler_handle = Some(handle);

        INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
    } else {
        let mut scheduler_handle = data.scheduler_handle.lock().await;
        if let Some(handle) = scheduler_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        let mut scheduler = initialize_scheduler(ctx, data)?;

        let handle: JoinHandle<()> = tokio::spawn(async move {
            loop {
                scheduler.run_pending().await;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

        *scheduler_handle = Some(handle);
    }

    let current_song = Songs::last_played(&data.db).await;
    if let Ok(Some(current_song)) = current_song {
        ctx.set_activity(Some(ActivityData::listening(format!(
            "{} - {}",
            current_song.album, current_song.title
        ))));
    }

    Ok(())
}
