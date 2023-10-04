use chrono::{DateTime, NaiveDateTime, TimeZone};
use fred::{
    prelude::{RedisError, RedisErrorKind},
    types::{FromRedis, RedisValue},
};

pub(crate) type Error = JudeHarleyError;
pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum JudeHarleyError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    SqlxMigration(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Redis(#[from] fred::prelude::RedisError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    PathStripPrefix(#[from] std::path::StripPrefixError),
    #[error(transparent)]
    Id3(#[from] id3::Error),
    #[error(transparent)]
    AudioTags(#[from] audiotags::Error),
}

pub trait DiscordTimestamp {
    fn short_time(&self) -> String;
    fn long_time(&self) -> String;
    fn short_date(&self) -> String;
    fn long_date(&self) -> String;
    fn long_date_short_time(&self) -> String;
    fn long_date_with_dow_short_time(&self) -> String;
    fn relative_time(&self) -> String;
}

impl DiscordTimestamp for i64 {
    fn short_time(&self) -> String {
        format!("<t:{}:t>", self)
    }

    fn long_time(&self) -> String {
        format!("<t:{}:T>", self)
    }

    fn short_date(&self) -> String {
        format!("<t:{}:d>", self)
    }

    fn long_date(&self) -> String {
        format!("<t:{}:D>", self)
    }

    fn long_date_short_time(&self) -> String {
        format!("<t:{}:f>", self)
    }

    fn long_date_with_dow_short_time(&self) -> String {
        format!("<t:{}:F>", self)
    }

    fn relative_time(&self) -> String {
        format!("<t:{}:R>", self)
    }
}

impl DiscordTimestamp for NaiveDateTime {
    fn short_time(&self) -> String {
        format!("<t:{}:t>", self.timestamp())
    }

    fn long_time(&self) -> String {
        format!("<t:{}:T>", self.timestamp())
    }

    fn short_date(&self) -> String {
        format!("<t:{}:d>", self.timestamp())
    }

    fn long_date(&self) -> String {
        format!("<t:{}:D>", self.timestamp())
    }

    fn long_date_short_time(&self) -> String {
        format!("<t:{}:f>", self.timestamp())
    }

    fn long_date_with_dow_short_time(&self) -> String {
        format!("<t:{}:F>", self.timestamp())
    }

    fn relative_time(&self) -> String {
        format!("<t:{}:R>", self.timestamp())
    }
}

impl<Tz> DiscordTimestamp for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn short_time(&self) -> String {
        format!("<t:{}:t>", self.timestamp())
    }

    fn long_time(&self) -> String {
        format!("<t:{}:T>", self.timestamp())
    }

    fn short_date(&self) -> String {
        format!("<t:{}:d>", self.timestamp())
    }

    fn long_date(&self) -> String {
        format!("<t:{}:D>", self.timestamp())
    }

    fn long_date_short_time(&self) -> String {
        format!("<t:{}:f>", self.timestamp())
    }

    fn long_date_with_dow_short_time(&self) -> String {
        format!("<t:{}:F>", self.timestamp())
    }

    fn relative_time(&self) -> String {
        format!("<t:{}:R>", self.timestamp())
    }
}

pub struct W<T>(pub T);

pub trait Wrappable {
    fn wrap(self) -> W<Self>
    where
        Self: Sized,
    {
        W(self)
    }
}

impl<T> Wrappable for T where T: Sized {}

impl<T> std::ops::Deref for W<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for W<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl TryFrom<W<chrono::NaiveDateTime>> for RedisValue {
    type Error = JudeHarleyError;

    fn try_from(value: W<chrono::NaiveDateTime>) -> Result<Self> {
        Ok(RedisValue::Integer(value.0.timestamp()))
    }
}

impl FromRedis for W<chrono::NaiveDateTime> {
    fn from_value(value: fred::types::RedisValue) -> std::result::Result<Self, RedisError> {
        if let fred::types::RedisValue::Integer(i) = value {
            Ok(W(chrono::NaiveDateTime::from_timestamp_opt(i, 0).unwrap()))
        } else {
            Err(fred::prelude::RedisError::new(
                RedisErrorKind::Parse,
                "invalid value",
            ))
        }
    }
}
