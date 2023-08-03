use std::net::ToSocketAddrs;

use commands::help::help;
use fred::{
    clients::SubscriberClient,
    prelude::{ClientLike, PubsubInterface, RedisClient},
    types::{PerformanceConfig, ReconnectPolicy, RedisConfig, RedisValue},
};
use poise::serenity_prelude::Activity;
use sqlx::postgres::PgPoolOptions;
use tracing::{debug, info};
use tracing_unwrap::{OptionExt, ResultExt};

use crate::{
    commands::{
        currency::{boondollars, pay, pay_menu},
        minigames::slots,
        songs::song_request,
        version::version,
        youtube::{link_youtube, unlink_youtube},
    },
    prelude::*,
};

mod app_config;
mod commands;
mod db;
mod event_handlers;
mod prelude;
mod telnet_communication;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Loading config from environment...");
    let config = app_config::AppConfig::from_env();
    let commands = vec![
        help(),
        song_request(),
        link_youtube(),
        unlink_youtube(),
        version(),
        boondollars(),
        pay(),
        pay_menu(),
        slots(),
    ];
    info!("Loading {} commands...", commands.len());

    info!("Connecting to database...");
    tracing::debug!("Database URL: {}", config.database_url);
    let db = PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .expect_or_log("failed to connect to database");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect_or_log("failed to run migrations");

    info!("Connecting to Liquidsoap...");
    let telnet = loop {
        let telnet = telnet::Telnet::connect(
            (config.liquidsoap.host.to_owned(), config.liquidsoap.port),
            256,
        );
        if let Err(e) = telnet {
            tracing::error!(
                "Failed to connect to Liquidsoap, reconnecting in 5 seconds: {}",
                e
            );
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        } else {
            break telnet.unwrap();
        }
    };

    info!("Connecting to Redis...");
    let redis_config = RedisConfig::from_url(&config.redis_url).expect_or_log("invalid Redis URL");
    let policy = ReconnectPolicy::new_exponential(0, 100, 30_000, 2);
    let perf = PerformanceConfig::default();
    let redis_client = RedisClient::new(
        redis_config.clone(),
        Some(perf.clone()),
        Some(policy.clone()),
    );
    let subscriber_client = SubscriberClient::new(
        redis_config.clone(),
        Some(perf.clone()),
        Some(policy.clone()),
    );

    let mut redis_error_rx = redis_client.on_error();
    let mut redis_reconnect_rx = redis_client.on_reconnect();
    let mut subscriber_error_rx = subscriber_client.on_error();
    let mut subscriber_reconnect_rx = subscriber_client.on_reconnect();

    tokio::spawn(async move {
        while let Ok(error) = redis_error_rx.recv().await {
            tracing::error!("Redis error: {:?}", error);
        }
    });
    tokio::spawn(async move {
        while redis_reconnect_rx.recv().await.is_ok() {
            tracing::info!("Redis reconnected");
        }
    });
    tokio::spawn(async move {
        while let Ok(error) = subscriber_error_rx.recv().await {
            tracing::error!("Redis subscriber error: {:?}", error);
        }
    });
    tokio::spawn(async move {
        while subscriber_reconnect_rx.recv().await.is_ok() {
            tracing::info!("Redis subscriber reconnected");
        }
    });

    let connection_task = redis_client.connect();
    redis_client
        .wait_for_connect()
        .await
        .expect_or_log("failed to connect to Redis");
    let subscriber_task = subscriber_client.connect();
    subscriber_client
        .wait_for_connect()
        .await
        .expect_or_log("failed to connect to Redis subscriber");

    let manage_handle = subscriber_client.manage_subscriptions();
    subscriber_client
        .subscribe::<(), _>("byers:status")
        .await
        .expect_or_log("failed to subscribe");

    let context = Data {
        db,
        telnet: std::sync::Arc::new(tokio::sync::Mutex::new(telnet)),
        google_config: config.google,
        redis_client: redis_client.clone(),
        subscriber_client: subscriber_client.clone(),
    };

    let framework_builder = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            event_handler: |ctx, event, _framework, data| {
                Box::pin(async move {
                    debug!("Event received: {}", event.name());

                    if let poise::Event::Message { new_message } = event {
                        event_handlers::message::message_handler(new_message, data)
                            .await
                            .expect_or_log("Failed to handle message");
                    }

                    if let poise::Event::Ready { data_about_bot } = event {
                        info!("Connected as {}", data_about_bot.user.name);

                        info!("Spawning Redis subscriber message handler...");
                        let mut message_rx = data.subscriber_client.on_message();
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
                    }

                    Ok(())
                })
            },
            ..Default::default()
        })
        .token(config.discord_token)
        .intents(*INTENTS)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Starting up Byers...");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(context)
            })
        });

    let framework = framework_builder.build().await.unwrap_or_log();

    let shard_handler = framework.shard_manager().clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect_or_log("failed to install CTRL+C handler");

        info!("Shutting down...");
        shard_handler.lock().await.shutdown_all().await;
    });

    framework.start().await.unwrap_or_log();

    redis_client
        .quit()
        .await
        .expect_or_log("failed to quit Redis");
    let _ = connection_task.await;
    subscriber_client
        .quit()
        .await
        .expect_or_log("failed to quit Redis subscriber");
    let _ = manage_handle.await;
    let _ = subscriber_task.await;
}
