use crate::prelude::*;

/// Shows the current version of Byers
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    let version = env!("CARGO_PKG_VERSION");
    let changelog =
        "<https://github.com/LumiRadio/lumiRadio/blob/develop/CHANGELOG.md>".to_string();

    ctx.say(format!("Byers is currently running version v{version}. You can view the changelog for this version at {changelog}."))
        .await?;

    Ok(())
}
