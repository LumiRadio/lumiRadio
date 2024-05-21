use poise::{
    serenity_prelude::{AutocompleteChoice, CreateEmbed, User},
    CreateReply,
};
use tracing_unwrap::ResultExt;

use crate::prelude::*;
use judeharley::{controllers::users::UpdateParams, Decimal, SlcbCurrency, Users};

pub async fn autocomplete_channels(
    ctx: ApplicationContext<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    let data = ctx.data;

    // AutocompleteChoice {
    //     name: format!(
    //         "{} (Hours: {}, Points: {})",
    //         user.username, user.hours, user.points
    //     ),
    //     value: user.id,
    // }
    SlcbCurrency::search(partial, &data.db)
        .await
        .expect_or_log("Failed to fetch possible channels")
        .into_iter()
        .take(20)
        .map(|user| {
            AutocompleteChoice::new(
                format!(
                    "{} (Hours: {}, Points: {})",
                    user.username, user.hours, user.points
                ),
                user.id,
            )
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

    let user = Users::get_or_insert(user.id.get(), &data.db).await?;
    if user.migrated {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("User already migrated")
                    .description("This user had their data already imported!"),
            ),
        )
        .await?;

        return Ok(());
    }

    let new_watch_time = user.watched_time + Decimal::from(hours);
    let new_boonbucks = user.boonbucks + points;
    user.update(
        UpdateParams {
            watched_time: Some(new_watch_time),
            boonbucks: Some(new_boonbucks as u32),
            migrated: Some(true),
            ..Default::default()
        },
        &data.db,
    )
    .await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Imported user data")
                .description(format!("Imported {} hours and {} points", hours, points)),
        ),
    )
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

    let user = Users::get_or_insert(user.id.get(), &data.db).await?;

    if user.migrated {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("User already migrated")
                    .description("This user had their data already imported!"),
            ),
        )
        .await?;

        return Ok(());
    }

    let Some(slcb_user) = SlcbCurrency::get(channel, &data.db).await? else {
        unreachable!("Autocomplete should prevent this from happening");
    };

    let new_watch_time = user.watched_time + Decimal::from(slcb_user.hours);
    let new_boonbucks = user.boonbucks + slcb_user.points;
    user.update(
        UpdateParams {
            watched_time: Some(new_watch_time),
            boonbucks: Some(new_boonbucks as u32),
            migrated: Some(true),
            ..Default::default()
        },
        &data.db,
    )
    .await?;

    ctx.send(
        CreateReply::default().embed(CreateEmbed::new().title("Imported user data").description(
            format!(
                "Imported {} hours and {} points from {}",
                slcb_user.hours, slcb_user.points, slcb_user.username
            ),
        )),
    )
    .await?;

    Ok(())
}
