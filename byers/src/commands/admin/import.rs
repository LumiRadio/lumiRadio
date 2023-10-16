use poise::{serenity_prelude::User, AutocompleteChoice};
use tracing_unwrap::ResultExt;

use crate::prelude::*;
use judeharley::{
    db::{DbSlcbUser, DbUser},
    BigDecimal,
};

pub async fn autocomplete_channels(
    ctx: ApplicationContext<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::AutocompleteChoice<i32>> {
    let data = ctx.data;

    DbSlcbUser::search_by_username(&data.db, partial)
        .await
        .expect_or_log("Failed to fetch possible channels")
        .into_iter()
        .take(20)
        .map(|user| AutocompleteChoice {
            name: format!(
                "{} (Hours: {}, Points: {})",
                user.username, user.hours, user.points
            ),
            value: user.id,
        })
}

/// Manually insert user data for a user
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn import_manually(
    ctx: ApplicationContext<'_>,
    #[description = "The user you want to import data for"] user: User,
    #[description = "The amount of hours to import"] hours: i32,
    #[description = "The amount of points to import"] points: i32,
) -> Result<(), Error> {
    let data = ctx.data();

    let mut user = DbUser::fetch_or_insert(&data.db, user.id.0 as i64).await?;
    if user.migrated {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("User already migrated")
                    .description("This user had their data already imported!")
            })
        })
        .await?;

        return Ok(());
    }

    user.watched_time += BigDecimal::from(hours);
    user.boonbucks += points;
    user.migrated = true;
    user.update(&data.db).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Imported user data")
                .description(format!("Imported {} hours and {} points", hours, points))
        })
    })
    .await?;

    Ok(())
}

/// Import user data from SLCB
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn import(
    ctx: ApplicationContext<'_>,
    #[description = "The user you want to import data for"] user: User,
    #[description = "The YouTube channel name to import data from"]
    #[autocomplete = "autocomplete_channels"]
    channel: i32,
) -> Result<(), Error> {
    let data = ctx.data();

    let mut user = DbUser::fetch_or_insert(&data.db, user.id.0 as i64).await?;

    if user.migrated {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("User already migrated")
                    .description("This user had their data already imported!")
            })
        })
        .await?;

        return Ok(());
    }

    let Some(slcb_user) = DbSlcbUser::fetch(&data.db, channel).await? else {
        unreachable!("Autocomplete should prevent this from happening");
    };
    user.watched_time += BigDecimal::from(slcb_user.hours);
    user.boonbucks += slcb_user.points;
    user.migrated = true;
    user.update(&data.db).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Imported user data").description(format!(
                "Imported {} hours and {} points from {}",
                slcb_user.hours, slcb_user.points, slcb_user.username
            ))
        })
    })
    .await?;

    Ok(())
}
