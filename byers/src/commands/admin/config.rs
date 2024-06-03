use poise::{
    serenity_prelude::{Channel, CreateEmbed, Role, UserId},
    CreateReply,
};

use crate::prelude::*;
use judeharley::{
    sea_orm::Set, Cans, ServerChannelConfig, ServerConfig, ServerRoleConfig, Users
};

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

    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;
    Cans::set(&user, can_count as i64, &data.db).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Can Count Set")
                .description(format!("Can count set to {}", can_count)),
        ),
    )
    .await?;

    Ok(())
}

/// Sets the quest roll
#[poise::command(slash_command, owners_only, ephemeral, guild_only)]
pub async fn set_quest_roll(ctx: ApplicationContext<'_>, roll: i32) -> Result<(), Error> {
    let data = ctx.data;

    let server_config =
        ServerConfig::get_or_insert(ctx.guild_id().unwrap().get(), &data.db).await?;
    server_config
        .update(
            judeharley::entities::server_config::ActiveModel {
                dice_roll: Set(roll),
                ..Default::default()
            },
            &data.db,
        )
        .await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Quest Roll Set")
                .description(format!("Quest roll set to {}", roll)),
        ),
    )
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

    let role_config =
        ServerRoleConfig::get_or_insert(role.id.get(), guild_id.get(), hours as u32, &data.db)
            .await?;
    role_config
        .update(
            judeharley::entities::server_role_config::ActiveModel {
                minimum_hours: Set(hours),
                ..Default::default()
            },
            &data.db,
        )
        .await?;
    let handle = ctx
        .send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Role Configured")
                    .description("Applying the roles for all users...")
                    .field("Role", &role.name, true)
                    .field("Minimum watch time", format!("{} hours", hours), true),
            ),
        )
        .await?;

    let users = Users::get_with_at_least_n_hours(hours, &data.db).await?;
    for user in users {
        let user_id = UserId::new(user.id as u64);
        let member = guild_id.member(&ctx.serenity_context(), user_id).await?;
        if member.roles.contains(&role.id) {
            continue;
        }

        if let Err(e) = member.add_role(&ctx.serenity_context(), role.id).await {
            tracing::error!("Failed to add role to user: {}", e);
        }
    }

    handle
        .edit(
            poise::Context::Application(ctx),
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Role Configured")
                    .description("All users have been updated")
                    .field("Role", &role.name, true)
                    .field("Minimum watch time", format!("{} hours", hours), true),
            ),
        )
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

    ServerRoleConfig::delete_by_role(role.id.get(), guild_id.get(), &data.db).await?;
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Role Config Deleted")
                .description("The role config has been deleted")
                .field("Role", &role.name, true),
        ),
    )
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
    #[description = "Remind people to hydrate in here"] hydration_reminder: bool,
) -> Result<(), Error> {
    let data = ctx.data;

    let channel_config = ServerChannelConfig::get_or_insert(channel.id().get(), &data.db).await?;
    

    channel_config.update(judeharley::entities::server_channel_config::ActiveModel {
        allow_point_accumulation: Set(allow_point_accumulation),
        allow_watch_time_accumulation: Set(allow_watch_time_accumulation),
        hydration_reminder: Set(hydration_reminder),
        ..Default::default()
    }, &data.db).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Channel Configured")
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
                .field(
                    "Remind people to hydrate",
                    hydration_reminder.to_string(),
                    true,
                ),
        ),
    )
    .await?;

    Ok(())
}
