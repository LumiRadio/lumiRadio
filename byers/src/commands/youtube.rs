use std::time::Duration;

use poise::{
    serenity_prelude::{CreateActionRow, CreateButton, CreateEmbed},
    CreateReply,
};
use situwaition::{
    runtime::AsyncWaiter, SituwaitionError, SituwaitionOptsBuilder, TokioAsyncSituwaition,
};
use tracing::error;

use crate::{event_handlers::message::update_activity, prelude::*};
use judeharley::{controllers::users::UpdateParams, Decimal, SlcbCurrency, Users};

/// Commands related to importing data from YouTube
#[poise::command(slash_command, subcommands("link"))]
pub async fn youtube(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum YoutubeError {
    #[error("No channel found")]
    NoChannelFound,
    #[error(transparent)]
    Jude(#[from] judeharley::JudeHarleyError),
}

/// Link your YouTube channel to your Discord account
#[poise::command(slash_command, ephemeral)]
pub async fn link(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data();

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;

    if user.migrated {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Already migrated")
                        .description("You have already migrated your account!"),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    let handle = ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Link your YouTube channel")
                    .description(r#"In order to link your YouTube channel with the bot, you will need to link your YouTube account with your Discord account.
To do that, go into your Settings, then "Connections" and then add your YouTube account to your Discord account. **Please make sure that your YouTube account name is the same as when you last chatted on the radio!**

After that, please press the **Log In** button below and complete the steps.
Once you have completed the steps, this message will update and prompt you to select the channel you want to import data from.
This relies on your channel name! If you have changed your channel name, please change it back to the old one, link your account and THEN log in with the button.

If you don't remember your old YouTube name or you no longer have access to your YouTube account, please message <@108693106194399232> about it!"#)
            )
            .components(vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new_link("https://discord.lumirad.io/oauth2/login")
                        .label("Log In")
                        .emoji('🔗'),
                ]),
            ])
    ).await?;

    let linked_channels = AsyncWaiter::with_opts(
        || async {
            let connected_channels = user
                .linked_channels(&ctx.data().db)
                .await
                .map_err(Into::<YoutubeError>::into)?;

            if connected_channels.is_empty() {
                return Err(YoutubeError::NoChannelFound);
            }

            Ok(connected_channels)
        },
        SituwaitionOptsBuilder::default()
            .timeout(Duration::from_secs(120))
            .check_interval(Duration::from_secs(1))
            .build()
            .unwrap(),
    )
    .exec()
    .await;

    // b.embed(|e| {
    //     e.title("No channels found")
    //         .description("No channels found. Please make sure you have linked your YouTube account with your Discord account!")
    // })
    // .components(|c| c)
    let channels = match linked_channels {
        Ok(c) => c,
        Err(SituwaitionError::TimeoutError(YoutubeError::NoChannelFound)) => {
            handle.edit(
                poise::Context::Application(ctx),
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title("No channels found")
                            .description("No channels found. Please make sure you have linked your YouTube account with your Discord account!"),
                    )
                    .components(vec![]),
            ).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Failed to fetch linked channels: {}", e);
            // b.embed(|e| {
            //     e.title("Failed to fetch linked channels")
            //         .description("Failed to fetch linked channels. Please try again later!")
            // })
            // .components(|c| c)
            handle
                .edit(
                    poise::Context::Application(ctx),
                    CreateReply::default()
                        .embed(
                            CreateEmbed::new()
                                .title("Failed to fetch linked channels")
                                .description(
                                    "Failed to fetch linked channels. Please try again later!",
                                ),
                        )
                        .components(vec![]),
                )
                .await?;
            return Ok(());
        }
    };

    let mut slcb_account = None;
    for youtube_channel in channels {
        if let Some(account) =
            SlcbCurrency::get_by_user_id(&youtube_channel.youtube_channel_id, &data.db).await?
        {
            slcb_account = Some(account);
            break;
        }
    }

    let Some(slcb_account) = slcb_account else {
        handle.edit(
            poise::Context::Application(ctx),
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("No importable channels found")
                        .description("No importable channels found. Please make sure you have linked your YouTube account with your Discord account! If it still doesn't show up, please message <@108693106194399232> about it!")
                )
                .components(vec![]),
        ).await?;
        return Ok(());
    };

    let new_watch_time = user.watched_time + Decimal::from(slcb_account.hours);
    let new_boonbucks = user.boonbucks as u32 + slcb_account.points as u32;
    user.update(
        UpdateParams {
            watched_time: Some(new_watch_time),
            boonbucks: Some(new_boonbucks),
            migrated: Some(true),
            ..Default::default()
        },
        &data.db,
    )
    .await?;

    handle
        .edit(
            Context::Application(ctx),
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Successfully imported data!")
                        .description(format!(
                            "Successfully imported data from {}!",
                            slcb_account.username
                        )),
                )
                .components(vec![]),
        )
        .await?;

    Ok(())
}
