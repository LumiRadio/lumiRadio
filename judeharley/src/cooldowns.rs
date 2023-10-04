use std::fmt::Display;

use chrono::NaiveDateTime;
use fred::{pool::RedisPool, prelude::KeysInterface};

use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct UserCooldownKey<'a> {
    pub user_id: i64,
    pub key: &'a str,
}

impl<'a> UserCooldownKey<'a> {
    pub fn new(user_id: i64, key: &'a str) -> Self {
        Self { user_id, key }
    }

    pub fn to_global(self) -> GlobalCooldownKey<'a> {
        GlobalCooldownKey::new(self.key)
    }
}

impl<'a> Display for UserCooldownKey<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("cooldown:{}:{}", self.user_id, self.key))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalCooldownKey<'a> {
    pub key: &'a str,
}

impl<'a> GlobalCooldownKey<'a> {
    pub fn new(key: &'a str) -> Self {
        Self { key }
    }

    pub fn to_user(self, user_id: i64) -> UserCooldownKey<'a> {
        UserCooldownKey::new(user_id, self.key)
    }
}

impl<'a> Display for GlobalCooldownKey<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("cooldown:{}", self.key))
    }
}

pub trait CooldownKey
where
    Self: Display,
{
}

impl<'a> CooldownKey for UserCooldownKey<'a> {}
impl<'a> CooldownKey for GlobalCooldownKey<'a> {}

pub async fn is_on_cooldown<C>(pool: &RedisPool, key: C) -> Result<Option<NaiveDateTime>>
where
    C: CooldownKey + Display,
{
    let key = key.to_string();
    let value: Option<String> = pool.get(&key).await?;

    let Some(value) = value else {
        return Ok(None);
    };

    let value: i64 = value.parse()?;
    let over = NaiveDateTime::from_timestamp_opt(value, 0).unwrap();
    let now = chrono::Utc::now().naive_utc();

    if over < now {
        return Ok(None);
    }

    Ok(Some(over))
}

pub async fn set_cooldown<C>(pool: &RedisPool, key: C, expires_in: i64) -> Result<()>
where
    C: CooldownKey + Display,
{
    let key = key.to_string();

    pool.set(
        &key,
        (chrono::Utc::now() + chrono::Duration::seconds(expires_in))
            .timestamp()
            .to_string(),
        Some(fred::types::Expiration::EX(expires_in)),
        None,
        false,
    )
    .await?;

    Ok(())
}
