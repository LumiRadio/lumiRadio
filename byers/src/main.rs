use std::str::FromStr;

use fred::prelude::{ClientLike, PubsubInterface};
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
    oauth2::oauth2_server,
    prelude::*,
};
use judeharley::communication::ByersUnixStream;

mod app_config;
mod commands;
mod event_handlers;
mod oauth2;
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
    let subscriber_client = judeharley::subscriber_client(&config.redis_url);

    let mut subscriber_error_rx = subscriber_client.on_error();
    let mut subscriber_reconnect_rx = subscriber_client.on_reconnect();

    let mut redis_error_rx = redis_pool.on_error();
    let mut redis_reconnect_rx = redis_pool.on_reconnect();

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

    let _ = redis_pool.connect();
    redis_pool
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
        db: db.clone(),
        comms: std::sync::Arc::new(tokio::sync::Mutex::new(
            ByersUnixStream::new().await.unwrap(),
        )),
        redis_pool: redis_pool.clone(),
        redis_subscriber: subscriber_client.clone(),
        emoji: config.discord.emoji.clone(),
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
            pre_command: |ctx| {
                Box::pin(async move {
                    sentry::add_breadcrumb(BreadcrumbableContext(ctx).as_breadcrumbs().await);
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

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let webserver_handle = tokio::spawn(oauth2_server(
        config.secret.clone(),
        db,
        redis_pool.clone(),
        config.discord,
        rx,
    ));

    let shard_handler = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect_or_log("failed to install CTRL+C handler");

        info!("Shutting down...");
        shard_handler.shutdown_all().await;
        tx.send(()).expect_or_log("failed to send shutdown signal");
        let _ = webserver_handle.await;
    });

    client.start().await.expect_or_log("failed to start client");

    redis_pool.quit_pool().await;
    subscriber_client
        .quit()
        .await
        .expect_or_log("failed to quit Redis subscriber client");

    let _ = manage_handle.await;
    let _ = subscriber_task.await;
}
