use std::net::ToSocketAddrs;

use commands::help::help;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_unwrap::{ResultExt, OptionExt};

use crate::{prelude::*, commands::{request::song_request, youtube::link_youtube}};

mod app_config;
mod prelude;
mod commands;
mod telnet_communication;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Loading config from environment...");
    let config = app_config::AppConfig::from_env();
    let commands = vec![help(), song_request(), link_youtube()];
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

    // let telnet = mini_telnet::Telnet::builder()
    //     .connect_timeout(std::time::Duration::from_secs(5))
    //     .timeout(std::time::Duration::from_secs(5))
    //     .connect(&config.liquidsoap)
    //     .await
    //     .expect_or_log("failed to connect to liquidsoap");
    let telnet = telnet::Telnet::connect((config.liquidsoap.host, config.liquidsoap.port), 256).expect_or_log("Failed to connect to liquidsoap");

    let context = Data {
        db,
        telnet: std::sync::Arc::new(tokio::sync::Mutex::new(telnet)),
        google_config: config.google,
    };

    let framework_builder = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .token(config.discord_token)
        .intents(INTENTS)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Starting up Byers...");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(context)
            })
        });
    
    let framework = framework_builder
        .build()
        .await
        .unwrap_or_log();

    let shard_handler = framework.shard_manager().clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect_or_log("failed to install CTRL+C handler");

        info!("Shutting down...");
        shard_handler.lock().await.shutdown_all().await;
    });

    framework.start().await.unwrap_or_log();
}

