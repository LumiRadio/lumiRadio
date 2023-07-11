use std::{sync::Arc, time::Duration};

use google_youtube3::oauth2::authenticator_delegate::DeviceFlowDelegate;
use poise::serenity_prelude::{self as serenity, ApplicationCommandInteraction};
use sqlx::types::BigDecimal;
use tracing_unwrap::{OptionExt, ResultExt};

use crate::{
    db::{DbSlcbUser, DbUser},
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

/// Links your YouTube channel with your Discord account.
#[poise::command(slash_command)]
pub async fn link_youtube(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let user_data = DbUser::fetch(&data.db, ctx.author().id.0 as i64)
        .await
        .expect_or_log("Failed to fetch user data");

    if let Some(user_data) = user_data {
        if user_data.youtube_channel_id.is_some() {
            ctx.send(|m| {
                m.content("You already have a YouTube channel linked.")
                    .ephemeral(true)
            })
            .await?;
            return Ok(());
        }
    }

    let secret = google_youtube3::oauth2::ApplicationSecret {
        client_id: data.google_config.client_id.clone(),
        client_secret: data.google_config.client_secret.clone(),
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        ..Default::default()
    };

    let ctx = match ctx {
        Context::Application(ctx) => ctx,
        _ => unreachable!(),
    };

    ctx.defer_ephemeral().await?;

    let interaction = match ctx.interaction {
        poise::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(cmd) => cmd,
        _ => unreachable!(),
    }
    .clone();
    let serenity_http = ctx.serenity_context().http.clone();

    let auth = google_youtube3::oauth2::DeviceFlowAuthenticator::builder(secret)
        .flow_delegate(Box::new(DeviceFlowDiscordDelegate {
            interaction,
            http: serenity_http,
        }))
        .build()
        .await
        .expect_or_log("Failed to build authenticator");

    let hub = google_youtube3::YouTube::new(
        google_youtube3::hyper::Client::builder().build(
            google_youtube3::hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    );
    let result = hub
        .channels()
        .list(&vec!["snippet".to_string()])
        .mine(true)
        .doit()
        .await;
    if let Err(e) = result {
        ctx.send(|m| {
            m.content(format!("Failed to link YouTube account: {}", e))
                .ephemeral(true)
        })
        .await?;
        return Ok(());
    }
    let (_, channel_list) = result.unwrap();
    let items = channel_list.items.unwrap();
    let channel = items.first();
    if channel.is_none() {
        ctx.send(|m| {
            m.content("Failed to link YouTube account: no channel found. Please try again and make sure you selected the correct account.")
                .ephemeral(true)
        }).await?;
        return Ok(());
    }
    let channel = channel.unwrap();
    let channel_id = channel.id.as_ref().unwrap();

    let user_id = ctx.interaction.user().id.0 as i64;

    let mut user_data = DbUser {
        id: user_id,
        youtube_channel_id: Some(channel_id.clone()),
        ..Default::default()
    };
    user_data
        .upsert(&data.db)
        .await
        .expect_or_log("Failed to upsert user");

    ctx.send(|m| {
        m.content(format!(
            "Successfully linked YouTube account **{}**. I will now try to import data from the old bot.",
            channel_id
        ))
        .ephemeral(true)
    })
    .await?;

    let channel_name = &channel.snippet.as_ref().unwrap().title;
    if channel_name.is_none() {
        ctx.send(|m| {
            m.content("Failed to import channel data: no channel name found. Please unlink your account with `/unlink_youtube` and try again. Make sure you select the correct account!")
                .ephemeral(true)
        }).await?;
        return Ok(());
    }
    let channel_name = channel_name.as_ref().unwrap();
    let possible_channels = DbSlcbUser::fetch_by_username(&data.db, channel_name)
        .await
        .expect_or_log("Failed to fetch SLCB user");

    let (slcb_channel, has_interaction) = if possible_channels.is_empty() {
        ctx.send(|m| {
            m.content(r#"No channel found to import. This could have multiple reasons:

            * You haven't spoken in the old radio (so there is no data to import)
            * You have changed your name on YouTube (please change your YouTube name back to the old one and try again. If you don't want to do that, ask cozyGalvinism to import your data manually)
            * You selected the wrong account when linking (please unlink using `/unlink_youtube` and try again)
            "#)
                .ephemeral(true)
        }).await?;
        return Ok(());
    } else if possible_channels.len() > 1 {
        let possible_channels_clone = possible_channels.clone();
        let handle = ctx
            .send(|m| {
                m.content("Please choose the channel you want to import data from:")
                    .ephemeral(true)
                    .components(|c| {
                        c.create_action_row(|ar| {
                            ar.create_select_menu(|sm| {
                                sm.options(|o| {
                                    for channel in possible_channels_clone {
                                        let mut option =
                                            serenity::CreateSelectMenuOption::default();
                                        option
                                            .label(&channel.username)
                                            .value(&channel.id.to_string())
                                            .description(format!(
                                                "Hours: {}, Points: {}",
                                                channel.hours, channel.points
                                            ));
                                        o.add_option(option);
                                    }
                                    o
                                })
                                .custom_id(format!("import_channel_{user_id}"))
                            })
                        })
                    })
            })
            .await?;
        let message = handle
            .message()
            .await
            .expect_or_log("Failed to fetch message");
        let interaction = match message
            .await_component_interaction(ctx.serenity_context())
            .timeout(Duration::from_secs(60 * 3))
            .await
        {
            None => {
                message
                    .reply(ctx.serenity_context(), "Timed out. Please try again.")
                    .await?;
                return Ok(());
            }
            Some(i) => i,
        };

        let selected_channel_id = interaction
            .data
            .values
            .first()
            .unwrap()
            .parse::<i32>()
            .expect_or_log("Failed to parse selected channel ID");

        interaction
            .create_interaction_response(ctx.serenity_context(), |r| {
                r.kind(serenity::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.content("Importing data now...").ephemeral(true)
                    })
            })
            .await?;

        (
            possible_channels
                .into_iter()
                .find(|c| c.id == selected_channel_id)
                .unwrap(),
            Some(interaction),
        )
    } else {
        (possible_channels.into_iter().next().unwrap(), None)
    };

    // update the channel hours and points (old hours + new hours, old points + new points)
    user_data.watched_time += BigDecimal::from(slcb_channel.hours);
    user_data.boonbucks += slcb_channel.points;

    user_data
        .update(&data.db)
        .await
        .expect_or_log("Failed to update user");

    match has_interaction {
        Some(interaction) => {
            interaction
                .edit_original_interaction_response(ctx.serenity_context(), |r| {
                    r.content("Successfully imported data!")
                })
                .await?;
        }
        None => {
            ctx.send(|m| m.content("Successfully imported data!").ephemeral(true))
                .await?;
        }
    }

    Ok(())
}

/// Unlink your YouTube account from your Discord account.
#[poise::command(slash_command)]
pub async fn unlink_youtube(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let user_data = DbUser::fetch(&data.db, ctx.author().id.0 as i64)
        .await
        .expect_or_log("Failed to fetch user data");

    if user_data.is_none() {
        ctx.send(|m| {
            m.content("You haven't linked your YouTube account yet.")
                .ephemeral(true)
        })
        .await?;
        return Ok(());
    }
    let user_data = user_data.unwrap();

    if user_data.youtube_channel_id.is_none() {
        ctx.send(|m| {
            m.content("You haven't linked your YouTube account yet.")
                .ephemeral(true)
        })
        .await?;
        return Ok(());
    }

    sqlx::query!(
        "UPDATE users SET youtube_channel_id = NULL WHERE id = $1",
        ctx.author().id.0 as i64
    )
    .execute(&data.db)
    .await
    .expect_or_log("Failed to unlink user");

    ctx.send(|m| {
        m.content("Successfully unlinked your YouTube account.")
            .ephemeral(true)
    })
    .await?;

    Ok(())
}
