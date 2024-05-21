use crate::event_handlers::message::update_activity;
use crate::prelude::*;
use fred::prelude::{Expiration, KeysInterface};
use judeharley::{sea_orm::DatabaseConnection, Cans, DiscordTimestamp, Users};
use poise::{serenity_prelude::CreateEmbed, CreateReply};

/// Adds... things
#[poise::command(slash_command, subcommands("can", "bear", "john"), subcommand_required)]
pub async fn add(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

fn can_name(prefix: &str, number_of_cans: i64) -> String {
    match number_of_cans {
        (0..=49_999) => format!("{prefix} Town"),
        (50_000..=999_999) => format!("{prefix} City"),
        (1_000_000..=49_999_999) => format!("{prefix} Country"),
        (50_000_000..=99_999_999) => format!("{prefix} Continent"),
        (100_000_000..=999_999_999) => format!("{prefix} Planet"),
        (1_000_000_000..=4_999_999_999) => format!("{prefix} Galaxy"),
        (5_000_000_000..=9_999_999_999) => format!("{prefix} Universe"),
        _ => format!("{prefix}finity"),
    }
}

async fn addcan_action(ctx: Context<'_>) -> Result<(), Error> {
    update_activity(ctx.data(), ctx.author().id, ctx.channel_id()).await?;

    let can_count = Cans::count(&ctx.data().db).await?;
    let can_town_name = can_name("Can", can_count);
    if ctx
        .data()
        .redis_pool
        .get::<Option<String>, _>("can")
        .await?
        .is_some()
    {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title(&can_town_name)
                        .description("Woah, slow down there! Rome wasn't built in a day!"),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }
    ctx.data()
        .redis_pool
        .set("can", "true", Some(Expiration::EX(35)), None, false)
        .await?;

    add_can(&ctx.data().db, ctx.author().id.get()).await?;

    let can_count = Cans::count(&ctx.data().db).await?;
    let can_town_name = can_name("Can", can_count);
    let now_in_35_seconds = chrono::Utc::now() + chrono::Duration::seconds(35);
    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title(&can_town_name)
                    .description(format!(
                        "You place a can in {can_town_name}. There's now {can_count} cans. Someone can add another {}.",
                        now_in_35_seconds.relative_time()
                    )),
            )
    ).await?;

    Ok(())
}

async fn addbear_action(ctx: Context<'_>) -> Result<(), Error> {
    update_activity(ctx.data(), ctx.author().id, ctx.channel_id()).await?;

    let can_count = Cans::count(&ctx.data().db).await?;
    let can_town_name = can_name("Bear", can_count);
    if ctx
        .data()
        .redis_pool
        .get::<Option<String>, _>("can")
        .await?
        .is_some()
    {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title(&can_town_name)
                        .description("Woah, slow down there! Rome wasn't built in a day!"),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }
    ctx.data()
        .redis_pool
        .set("can", "true", Some(Expiration::EX(35)), None, false)
        .await?;

    add_can(&ctx.data().db, ctx.author().id.get()).await?;

    let can_count = Cans::count(&ctx.data().db).await?;
    let can_town_name = can_name("Bear", can_count);
    let now_in_35_seconds = chrono::Utc::now() + chrono::Duration::seconds(35);
    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title(&can_town_name)
                    .description(format!(
                        "You place a bear in {can_town_name}. There's now {can_count} bears. Someone can add another {}.",
                        now_in_35_seconds.relative_time()
                    )),
            )
    ).await?;

    Ok(())
}

#[allow(unused_variables)]
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

#[allow(unused_variables)]
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

async fn add_can(db: &DatabaseConnection, user_id: u64) -> Result<(), Error> {
    let user = Users::get_or_insert(user_id, db).await?;
    Cans::insert(&user, true, db).await?;

    Ok(())
}

#[allow(unused_variables)]
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

#[allow(unused_variables)]
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
    update_activity(ctx.data(), ctx.author().id, ctx.channel_id()).await?;

    ctx.send(CreateReply::default().embed(CreateEmbed::new().title("no").description("just no")))
        .await?;

    Ok(())
}
