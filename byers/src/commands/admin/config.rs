use poise::serenity_prelude::{Channel, ChannelId, Role, UserId};

use crate::prelude::*;
use judeharley::db::{DbCan, DbServerChannelConfig, DbServerConfig, DbServerRoleConfig, DbUser};

/// Configuration-related commands
#[poise::command(
    slash_command,
    owners_only,
    ephemeral,
    subcommands(
        "manage_channel",
        "set_can_count",
        "set_quest_roll",
        "manage_role",
        "delete_role_config"
    ),
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

/// Configures a role that should be automatically granted based on the specified watch time
#[poise::command(slash_command, owners_only, ephemeral, guild_only)]
pub async fn manage_role(
    ctx: ApplicationContext<'_>,
    #[description = "Role to manage"] role: Role,
    #[description = "Minimum watch time"]
    #[min = 1]
    hours: i32,
) -> Result<(), Error> {
    let data = ctx.data;
    let guild_id = ctx.guild_id().unwrap();

    DbServerRoleConfig::upsert(&data.db, guild_id.0 as i64, role.id.0 as i64, hours).await?;
    let handle = ctx
        .send(|m| {
            m.embed(|e| {
                e.title("Role Configured")
                    .description("Applying the roles for all users...")
                    .field("Role", &role.name, true)
                    .field("Minimum watch time", format!("{} hours", hours), true)
            })
        })
        .await?;

    let users = DbUser::fetch_by_minimum_hours(&data.db, hours).await?;
    for user in users {
        let user_id = UserId(user.id as u64);
        let mut member = guild_id.member(&ctx.serenity_context(), user_id).await?;
        if member.roles.contains(&role.id) {
            continue;
        }

        if let Err(e) = member.add_role(&ctx.serenity_context(), role.id).await {
            tracing::error!("Failed to add role to user: {}", e);
        }
    }

    handle
        .edit(poise::Context::Application(ctx), |m| {
            m.embed(|e| {
                e.title("Role Configured")
                    .description("All users have been updated")
                    .field("Role", &role.name, true)
                    .field("Minimum watch time", format!("{} hours", hours), true)
            })
        })
        .await?;

    Ok(())
}

#[poise::command(slash_command, owners_only, ephemeral, guild_only)]
pub async fn delete_role_config(
    ctx: ApplicationContext<'_>,
    #[description = "The role to delete the config for"] role: Role,
) -> Result<(), Error> {
    let data = ctx.data;
    let guild_id = ctx.guild_id().unwrap();

    DbServerRoleConfig::delete_by_guild_role(&data.db, guild_id.0 as i64, role.id.0 as i64).await?;
    ctx.send(|m| {
        m.embed(|e| {
            e.title("Role Config Deleted")
                .description("The role config has been deleted")
                .field("Role", &role.name, true)
        })
    })
    .await?;

    Ok(())
}

/// Configures a channel for watchtime and point accumulation
#[poise::command(slash_command, owners_only, ephemeral, guild_only)]
pub async fn manage_channel(
    ctx: ApplicationContext<'_>,
    #[description = "Channel to manage"] channel: Channel,
    #[description = "Allow point accumulation"] allow_point_accumulation: bool,
    #[description = "Allow watch time accumulation"] allow_watch_time_accumulation: bool,
) -> Result<(), Error> {
    let data = ctx.data;

    let mut channel_config = DbServerChannelConfig::fetch_or_insert(
        &data.db,
        channel.id().0 as i64,
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
