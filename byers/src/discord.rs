use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::prelude::Error;

#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum DiscordConnectionVisibility {
    None = 0,
    Everyone = 1,
}

#[derive(Deserialize, Debug)]
pub struct DiscordConnection {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub revoked: Option<bool>,
    pub verified: bool,
    pub friend_sync: bool,
    pub show_activity: bool,
    pub two_way_link: bool,
    pub visibility: DiscordConnectionVisibility,
}

impl DiscordConnection {
    pub async fn fetch(token: &str) -> Result<Vec<DiscordConnection>, Error> {
        let client = reqwest::Client::new();

        client
            .get("https://discord.com/api/users/@me/connections")
            .bearer_auth(token)
            .send()
            .await?
            .json::<Vec<DiscordConnection>>()
            .await
            .map_err(Error::from)
    }
}

#[derive(Deserialize, Debug)]
pub struct MinimalDiscordUser {
    pub id: String,
    pub username: String,
}

impl MinimalDiscordUser {
    pub async fn fetch(token: &str) -> Result<MinimalDiscordUser, Error> {
        let client = reqwest::Client::new();

        client
            .get("https://discord.com/api/users/@me")
            .bearer_auth(token)
            .send()
            .await?
            .json::<MinimalDiscordUser>()
            .await
            .map_err(Error::from)
    }
}
