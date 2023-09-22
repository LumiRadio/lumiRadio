use crate::db::DbCan;
use crate::event_handlers::message::update_activity;
use crate::prelude::{ApplicationContext, Context, Error};
use fred::prelude::{Expiration, KeysInterface};
use sqlx::PgPool;

/// Adds... things
#[poise::command(slash_command, subcommands("can", "bear", "john"), subcommand_required)]
pub async fn add(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

async fn addcan_action(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        update_activity(ctx.data(), ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    if ctx
        .data()
        .redis_pool
        .get::<Option<String>, _>("can")
        .await?
        .is_some()
    {
        return Ok(());
    }
    ctx.data()
        .redis_pool
        .set("can", "true", Some(Expiration::EX(35)), None, false)
        .await?;

    add_can(&ctx.data().db, ctx.author().id.0).await?;

    let can_count = DbCan::count(&ctx.data().db).await?;
    ctx.send(|m| {
        m.embed(|e| {
            e.title("Can Town")
                .description(format!("You place a can in Can Town. There's now {} cans. Someone can add another in 35 seconds.", can_count))
        })
    }).await?;

    Ok(())
}

async fn addbear_action(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        update_activity(ctx.data(), ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    if ctx
        .data()
        .redis_pool
        .get::<Option<String>, _>("can")
        .await?
        .is_some()
    {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Bear Town")
                    .description("Woah, slow down there! Rome wasn't built in a day!")
            })
            .ephemeral(true)
        })
        .await?;

        return Ok(());
    }
    ctx.data()
        .redis_pool
        .set("can", "true", Some(Expiration::EX(35)), None, false)
        .await?;

    add_can(&ctx.data().db, ctx.author().id.0).await?;

    let can_count = DbCan::count(&ctx.data().db).await?;
    ctx.send(|m| {
        m.embed(|e| {
            e.title("Bear Town")
                .description(format!("You place a bear in Bear Town. There's now {} bears. Someone can add another in 35 seconds.", can_count))
        })
    }).await?;

    Ok(())
}

/// Add a can to can town
#[poise::command(prefix_command, slash_command)]
pub async fn addcan(
    ctx: Context<'_>,
    #[description = "A comment for adding the can"]
    #[rest]
    comment: Option<String>,
) -> Result<(), Error> {
    addcan_action(ctx).await
}

/// Add a... bear...? to bear town...?
#[poise::command(prefix_command, slash_command)]
pub async fn addbear(
    ctx: Context<'_>,
    #[description = "A comment for adding the can"]
    #[rest]
    comment: Option<String>,
) -> Result<(), Error> {
    addbear_action(ctx).await
}

async fn add_can(db: &PgPool, user_id: u64) -> Result<(), Error> {
    DbCan::add_one(db, user_id as i64, true).await?;

    Ok(())
}

/// Add a can to can town
#[poise::command(slash_command)]
pub async fn can(
    ctx: Context<'_>,
    #[description = "A comment for adding the can"]
    #[rest]
    comment: Option<String>,
) -> Result<(), Error> {
    addcan_action(ctx).await
}

/// Add a... bear...? to bear town...?
#[poise::command(slash_command)]
pub async fn bear(
    ctx: Context<'_>,
    #[description = "A comment for adding the can"]
    #[rest]
    comment: Option<String>,
) -> Result<(), Error> {
    addbear_action(ctx).await
}

/// no
#[poise::command(slash_command)]
pub async fn john(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        update_activity(ctx.data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    ctx.send(|m| m.embed(|e| e.title("no").description("just no")))
        .await?;

    Ok(())
}
