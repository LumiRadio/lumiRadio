use crate::event_handlers::message::update_activity;
use crate::prelude::*;

/// there is no help
#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    update_activity(ctx.data(), ctx.author().id, ctx.channel_id()).await?;

    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: r#"Use `/help [command]` for more info on a command."#,
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;

    Ok(())
}
