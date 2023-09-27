use std::{sync::Arc, time::Duration};

use google_youtube3::oauth2::authenticator_delegate::DeviceFlowDelegate;
use poise::serenity_prelude::{self as serenity, ApplicationCommandInteraction};
use situwaition::{runtime::AsyncWaiter, SituwaitionError, TokioAsyncSituwaition};
use sqlx::types::BigDecimal;
use tracing::error;
use tracing_unwrap::ResultExt;

use crate::{
    db::{DbSlcbUser, DbUser},
    event_handlers::message::update_activity,
    prelude::*,
};

#[derive(Clone)]
struct DeviceFlowDiscordDelegate {
    interaction: ApplicationCommandInteraction,
    http: Arc<serenity::Http>,
}

async fn send_user_url(
    device_auth_resp: &google_youtube3::oauth2::authenticator_delegate::DeviceAuthResponse,
    interaction: &ApplicationCommandInteraction,
    http: &serenity::Http,
) {
    let msg = format!(
        "Please go to {} and enter the code {}. You only need to do this once.",
        device_auth_resp.verification_uri, device_auth_resp.user_code
    );
    interaction
        .create_followup_message(http, |f| f.content(msg).ephemeral(true))
        .await
        .expect_or_log("Failed to send followup message");
}

impl DeviceFlowDelegate for DeviceFlowDiscordDelegate {
    fn present_user_code<'yt>(
        &'yt self,
        device_auth_resp: &'yt google_youtube3::oauth2::authenticator_delegate::DeviceAuthResponse,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'yt>> {
        Box::pin(send_user_url(
            device_auth_resp,
            &self.interaction,
            &self.http,
        ))
    }
}

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
    Sqlx(#[from] sqlx::Error),
}

/// Link your YouTube channel to your Discord account
#[poise::command(slash_command, ephemeral)]
pub async fn link(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data();

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let mut user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;

    if user.migrated {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Already migrated")
                    .description("You have already migrated your account!")
            })
        })
        .await?;

        return Ok(());
    }

    let handle = ctx.send(|m| {
        m.embed(|e| {
            e.title("Link your YouTube channel")
                .description(r#"In order to link your YouTube channel with the bot, you will need to link your YouTube account with your Discord account.
                To do that, go into your Settings, then "Connections" and then add your YouTube account to your Discord account. **Please make sure that your YouTube account name is the same as when you last chatted on the radio!**

                After that, please press the **Log In** button below and complete the steps.
                Once you have completed the steps, this message will update and prompt you to select the channel you want to import data from.
                This relies on your channel name! If you have changed your channel name, please change it back to the old one, link your account and THEN log in with the button.
                
                If you don't remember your old YouTube name or you no longer have access to your YouTube account, please message <@108693106194399232> about it!"#)
        })
        .components(|c| {
            c.create_action_row(|ar| {
                ar.create_button(|b| {
                    b.label("Log In")
                    .style(serenity::ButtonStyle::Link)
                    .emoji('🔗')
                    .url("https://discord.lumirad.io/oauth2/login")
                })
            })
        })
    }).await?;

    let linked_channels = AsyncWaiter::with_timeout(
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
        Duration::from_secs(120),
    )?
    .exec()
    .await;

    let channels = match linked_channels {
        Ok(c) => c,
        Err(SituwaitionError::TimeoutError(YoutubeError::NoChannelFound)) => {
            handle.edit(poise::Context::Application(ctx), |b| {
                b.embed(|e| {
                    e.title("No channels found")
                        .description("No channels found. Please make sure you have linked your YouTube account with your Discord account!")
                })
                .components(|c| c)
            }).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Failed to fetch linked channels: {}", e);
            handle
                .edit(poise::Context::Application(ctx), |b| {
                    b.embed(|e| {
                        e.title("Failed to fetch linked channels")
                            .description("Failed to fetch linked channels. Please try again later!")
                    })
                    .components(|c| c)
                })
                .await?;
            return Ok(());
        }
    };

    let mut slcb_account = None;
    for youtube_channel in channels {
        if let Some(account) =
            DbSlcbUser::fetch_by_user_id(&data.db, &youtube_channel.youtube_channel_id).await?
        {
            slcb_account = Some(account);
            break;
        }
    }

    let Some(slcb_account) = slcb_account else {
        handle.edit(poise::Context::Application(ctx), |b| {
            b.embed(|e| {
                e.title("No importable channels found")
                    .description("No importable channels found. Please make sure you have linked your YouTube account with your Discord account! If it still doesn't show up, please message <@108693106194399232> about it!")
            })
            .components(|c| c)
        }).await?;
        return Ok(());
    };

    user.watched_time += BigDecimal::from(slcb_account.hours);
    user.boonbucks += slcb_account.points;
    user.migrated = true;

    user.update(&data.db).await?;

    handle
        .edit(Context::Application(ctx), |b| {
            b.embed(|e| {
                e.title("Successfully imported data!").description(format!(
                    "Successfully imported data from {}!",
                    slcb_account.username
                ))
            })
            .components(|c| c)
        })
        .await?;

    Ok(())
}
