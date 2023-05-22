use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;

use crate::app_config::GoogleConfig;

pub const INTENTS: GatewayIntents = GatewayIntents::non_privileged();

pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
// pub type Telnet = std::sync::Arc<tokio::sync::Mutex<mini_telnet::Telnet>>;
pub type Telnet = std::sync::Arc<tokio::sync::Mutex<telnet::Telnet>>;

pub struct Data {
    pub db: sqlx::PgPool,
    pub telnet: Telnet,
    pub google_config: GoogleConfig
}