use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;

use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

use crate::app_config::GoogleConfig;
use judeharley::communication::{ByersUnixStream, LiquidsoapCommunication};

lazy_static! {
    pub static ref INTENTS: GatewayIntents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
}

pub type Context<'a, C = ByersUnixStream> = poise::Context<'a, Data<C>, Error>;
pub type ApplicationContext<'a, C = ByersUnixStream> =
    poise::ApplicationContext<'a, Data<C>, Error>;
pub type Error = anyhow::Error;

pub struct Data<C>
where
    C: LiquidsoapCommunication,
{
    pub db: judeharley::PgPool,
    pub comms: Arc<Mutex<C>>,
    pub google_config: GoogleConfig,
    pub redis_pool: fred::pool::RedisPool,
    pub redis_subscriber: fred::clients::SubscriberClient,
}
