use sqlx::PgPool;
use crate::db::DbCan;
use crate::prelude::{ApplicationContext, Error};

/// Adds... things
#[poise::command(slash_command, subcommands("can"), subcommand_required, global_cooldown = 35)]
pub async fn add(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

async fn add_can(db: &PgPool, user_id: u64) -> Result<(), Error> {
    DbCan::add_one(db, user_id as i64, true).await?;

    Ok(())
}

/// Add a can to can town
#[poise::command(slash_command)]
pub async fn can(ctx: ApplicationContext<'_>) -> Result<(), Error> {
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

/// Add a... bear...? to can town...?
#[poise::command(slash_command)]
pub async fn bear(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    add_can(&ctx.data().db, ctx.author().id.0).await?;

    let can_count = DbCan::count(&ctx.data().db).await?;
    ctx.send(|m| {
        m.embed(|e| {
            e.title("Can Town")
                .description(format!("You place a ~~bear~~ can in Can Town. There's now {} cans. Someone can add another in 35 seconds.", can_count))
        })
    }).await?;

    Ok(())
}

/// no
#[poise::command(slash_command)]
pub async fn john(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    ctx.send(|m| {
        m.embed(|e| {
            e.title("no")
                .description("just no")
        })
    }).await?;

    Ok(())
}