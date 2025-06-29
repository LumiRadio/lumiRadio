use std::time::Duration;

use migration::MigratorTrait;

pub use crate::prelude::*;
pub use sea_orm;
pub use sea_orm::entity::prelude::Decimal;

pub mod communication;
pub mod controllers;
pub mod cooldowns;
pub mod custom_entities;
pub mod discord;
pub mod entities;
pub mod prelude;

pub mod maintenance;

pub async fn migrate(db: &sea_orm::DatabaseConnection) -> Result<()> {
    migration::Migrator::up(db, None).await?;

    Ok(())
}

pub async fn connect_database(url: &str) -> Result<sea_orm::DatabaseConnection> {
    sea_orm::Database::connect(url).await.map_err(Into::into)
}

pub fn redis_pool(redis_url: &str) -> Result<fred::prelude::Pool> {
    use fred::prelude::*;

    let redis_config = Config::from_url(redis_url)?;
    let client = Builder::from_config(redis_config)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(5);
            config.tcp = TcpConfig {
                nodelay: Some(true),
                ..Default::default()
            };
        })
        .build_pool(5)?;

    Ok(client)
}

pub fn subscriber_client(redis_url: &str) -> Result<fred::clients::SubscriberClient> {
    use fred::prelude::*;

    let redis_config = Config::from_url(redis_url).expect("invalid Redis URL");
    let client = Builder::from_config(redis_config)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(5);
            config.tcp = TcpConfig {
                nodelay: Some(true),
                ..Default::default()
            };
        })
        .build_subscriber_client()?;

    Ok(client)
}
