use fred::{
    clients::SubscriberClient,
    pool::RedisPool,
    types::{PerformanceConfig, ReconnectPolicy, RedisConfig},
};
use sqlx::postgres::PgPoolOptions;

pub use crate::prelude::*;
pub use sqlx::{types::BigDecimal, PgPool};

pub mod communication;
pub mod cooldowns;
pub mod db;
pub mod discord;
pub mod prelude;

pub mod maintenance;

pub async fn migrate(db: &PgPool) -> Result<()> {
    sqlx::migrate!().run(db).await.map_err(Into::into)
}

pub async fn connect_database(url: &str) -> Result<PgPool> {
    let db = PgPoolOptions::new().connect(url).await?;

    Ok(db)
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
