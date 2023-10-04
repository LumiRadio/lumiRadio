use crate::event_handlers::message::update_activity;
use byers::prelude::*;

/// Shows the current version of Byers
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        update_activity(ctx.data(), ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let version = env!("CARGO_PKG_VERSION");
    let changelog =
        format!("<https://github.com/LumiRadio/lumiRadio/blob/develop/CHANGELOG.md#{version}>");

    ctx.say(format!("Byers is currently running version v{version}. You can view the changelog for this version at {changelog}."))
        .await?;

    Ok(())
}
