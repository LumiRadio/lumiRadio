use poise::serenity_prelude::ChannelId;

use crate::{
    db::{DbCan, DbServerChannelConfig, DbServerConfig},
    prelude::{ApplicationContext, Error},
};

/// Configuration-related commands
#[poise::command(
    slash_command,
    owners_only,
    ephemeral,
    subcommands("manage_channel", "set_can_count", "set_quest_roll"),
    subcommand_required
)]
pub async fn config(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// Sets the can count (this may remove cans from users)
#[poise::command(slash_command, owners_only, ephemeral)]
pub async fn set_can_count(ctx: ApplicationContext<'_>, can_count: i32) -> Result<(), Error> {
    let data = ctx.data;

    DbCan::set(&data.db, ctx.author().id.0 as i64, can_count).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Can Count Set")
                .description(format!("Can count set to {}", can_count))
        })
    })
    .await?;

    Ok(())
}

/// Sets the quest roll
#[poise::command(slash_command, owners_only, ephemeral, guild_only)]
pub async fn set_quest_roll(ctx: ApplicationContext<'_>, roll: i32) -> Result<(), Error> {
    let data = ctx.data;

    let mut server_config =
        DbServerConfig::fetch_or_insert(&data.db, ctx.guild_id().unwrap().0 as i64).await?;
    server_config.dice_roll = roll;
    server_config.update(&data.db).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Quest Roll Set")
                .description(format!("Quest roll set to {}", roll))
        })
    })
    .await?;

    Ok(())
}

/// Configures a channel for watchtime and point accumulation
#[poise::command(slash_command, owners_only, ephemeral, guild_only)]
pub async fn manage_channel(
    ctx: ApplicationContext<'_>,
    #[description = "Channel to manage"] channel: ChannelId,
    #[description = "Allow point accumulation"] allow_point_accumulation: bool,
    #[description = "Allow watch time accumulation"] allow_watch_time_accumulation: bool,
) -> Result<(), Error> {
    let data = ctx.data;

    let mut channel_config = DbServerChannelConfig::fetch_or_insert(
        &data.db,
        channel.0 as i64,
        ctx.guild_id().unwrap().0 as i64,
    )
    .await?;
    channel_config.allow_point_accumulation = allow_point_accumulation;
    channel_config.allow_watch_time_accumulation = allow_watch_time_accumulation;
    channel_config.update(&data.db).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Channel Configured")
                .field(
                    "Allow point accumulation",
                    allow_point_accumulation.to_string(),
                    true,
                )
                .field(
                    "Allow watch time accumulation",
                    allow_watch_time_accumulation.to_string(),
                    true,
                )
        })
    })
    .await?;

    Ok(())
}
