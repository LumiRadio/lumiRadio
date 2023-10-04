use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;

lazy_static! {
    pub static ref INTENTS: GatewayIntents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
}

pub type Context<'a, C = ByersUnixStream> = poise::Context<'a, Data<C>, Error>;
pub type ApplicationContext<'a, C = ByersUnixStream> =
    poise::ApplicationContext<'a, Data<C>, Error>;
