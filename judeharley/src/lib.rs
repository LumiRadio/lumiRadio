use fred::{
    clients::SubscriberClient,
    pool::RedisPool,
    types::{PerformanceConfig, ReconnectPolicy, RedisConfig},
};
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

pub fn redis_pool(redis_url: &str) -> Result<RedisPool> {
    let redis_config = RedisConfig::from_url(redis_url)?;
    let policy = ReconnectPolicy::new_exponential(0, 100, 30_000, 2);
    let perf = PerformanceConfig::default();
    let redis_pool = RedisPool::new(redis_config, Some(perf), Some(policy), 5)?;

    Ok(redis_pool)
}

pub fn subscriber_client(redis_url: &str) -> SubscriberClient {
    let redis_config = RedisConfig::from_url(redis_url).expect("invalid Redis URL");
    let policy = ReconnectPolicy::new_exponential(0, 100, 30_000, 2);
    let perf = PerformanceConfig::default();

    SubscriberClient::new(redis_config, Some(perf), Some(policy))
}
