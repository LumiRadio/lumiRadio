use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string or string-convertible value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}

fn yes_this_is_definitely_a_string<'de, D: Deserializer<'de>>(de: D) -> Result<String, D::Error> {
    de.deserialize_any(StringVisitor)
}

#[derive(Deserialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub bot_auth: BotAuthenticationConfig,
    pub jwt: JwtConfig,
    pub database_url: String,
    pub liquidsoap_socket: PathBuf,
}

impl Config {
    pub fn new() -> figment::Result<Self> {
        figment::Figment::new()
            .merge(figment::providers::Env::prefixed("CALIBORN__").split("__"))
            .extract()
    }
}

#[derive(Deserialize)]
pub struct DiscordConfig {
    #[serde(deserialize_with = "yes_this_is_definitely_a_string")]
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Deserialize)]
pub struct BotAuthenticationConfig {
    pub secret_key: String,
}

#[derive(Deserialize)]
pub struct JwtConfig {
    pub secret: String,
}
