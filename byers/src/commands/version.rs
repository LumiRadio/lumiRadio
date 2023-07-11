use crate::prelude::*;

#[poise::command(
    slash_command,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    required_bot_permissions = "SEND_MESSAGES"
)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    let version = env!("CARGO_PKG_VERSION");
    let changelog =
        format!("https://github.com/LumiRadio/lumiRadio/blob/develop/CHANGELOG.md#{version}");

    ctx.say(format!("Byers is currently running version v{version}. You can view the changelog for this version at {changelog}."))
        .await?;

    Ok(())
}
