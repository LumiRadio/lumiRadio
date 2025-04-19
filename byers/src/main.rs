use std::str::FromStr;

use fred::prelude::ClientLike;
use poise::serenity_prelude as serenity;
use poise::PrefixFrameworkOptions;
use tracing::{debug, info};
use tracing_unwrap::ResultExt;

use crate::{
    commands::{
        add_stuff::*,
        admin::{config::config as config_cmd, import::*, user::*, *},
        context::what_song,
        currency::*,
        help::*,
        listen, minigames,
        minigames::pvp::pvp_context,
        songs::*,
        version::*,
        youtube::*,
    },
    prelude::*,
};
use judeharley::communication::ByersUnixStream;

mod app_config;
mod commands;
mod event_handlers;
mod prelude;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Loading config from environment...");
    let config = crate::app_config::AppConfig::from_env();

    let _guard = if let Some(sentry_dsn) = &config.sentry_dsn {
        info!("Initializing Sentry...");
        let guard = sentry::init(sentry::ClientOptions {
            environment: Some(config.environment.clone().into()),
            dsn: Some(
                sentry::types::Dsn::from_str(sentry_dsn)
                    .expect_or_log("failed to parse Sentry DSN"),
            ),
            release: sentry::release_name!(),
            ..Default::default()
        });

        Some(guard)
    } else {
        None
    };

    let commands = vec![
        help(),
        song(),
        youtube(),
        version(),
        boondollars(),
        pay(),
        pay_menu(),
        admin(),
        import(),
        config_cmd(),
        user(),
        minigames::command(),
        add(),
        listen(),
        pvp_context(),
        addcan(),
        addbear(),
        what_song(),
    ];

    info!("Loading {} commands...", commands.len());

    info!("Connecting to database...");
    tracing::debug!("Database URL: {}", config.database_url);
    let db = judeharley::connect_database(&config.database_url)
        .await
        .expect_or_log("failed to connect to database");

    judeharley::migrate(&db)
        .await
        .expect_or_log("failed to migrate database");

    info!("Connecting to Redis...");
    let redis_pool =
        judeharley::redis_pool(&config.redis_url).expect_or_log("failed to create Redis pool");
    let handle = redis_pool.init().await.expect_or_log("failed to connect to Redis");

    let context = Data {
        db: db.clone(),
        comms: std::sync::Arc::new(tokio::sync::Mutex::new(
            ByersUnixStream::new().await.unwrap(),
        )),
        redis_pool: redis_pool.clone(),
        emoji: config.discord.emoji.clone(),
        scheduler_handle: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            event_handler: |ctx, event, _framework, data| {
                Box::pin(async move {
                    debug!("Event received: {}", event.snake_case_name());

                    if let serenity::FullEvent::Message { new_message } = event {
                        crate::event_handlers::message::message_handler(new_message, data)
                            .await
                            .expect_or_log("Failed to handle message");
                    }

                    if let serenity::FullEvent::Ready { data_about_bot } = event {
                        crate::event_handlers::ready::on_ready(ctx, data_about_bot, data).await?;
                    }

                    Ok(())
                })
            },
            on_error: |error| {
                Box::pin(async move {
                    crate::event_handlers::error::on_error(error)
                        .await
                        .expect_or_log("Failed to handle error");
                })
            },
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".to_string()),
                ignore_bots: true,
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Starting up Byers...");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(context)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(&config.discord_token, *INTENTS)
        .framework(framework)
        .await
        .expect_or_log("failed to create client");

    let shard_handler = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect_or_log("failed to install CTRL+C handler");

        info!("Shutting down...");
        shard_handler.shutdown_all().await;
    });

    client.start().await.expect_or_log("failed to start client");

    redis_pool.quit().await.expect_or_log("failed to quit Redis");
    handle.await
        .expect_or_log("failed to await join handle")
        .expect_or_log("failed to await Redis quit");
}
