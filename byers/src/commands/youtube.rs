use std::sync::Arc;

use google_youtube3::oauth2::authenticator_delegate::DeviceFlowDelegate;
use poise::serenity_prelude::{self as serenity, ApplicationCommandInteraction};
use tracing_unwrap::ResultExt;

use crate::prelude::*;

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

#[poise::command(slash_command)]
pub async fn link_youtube(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
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
    let result = hub.channels().list(&vec!["snippet".to_string()]).mine(true).doit().await;
    if let Err(e) = result {
        ctx.send(|m| {
            m.content(format!("Failed to link YouTube account: {}", e))
                .ephemeral(true)
        }).await?;
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
    sqlx::query!(
        "INSERT INTO users (id, youtube_channel_id) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET youtube_channel_id = $2",
        user_id,
        channel_id
    )
    .execute(&data.db)
    .await
    .expect_or_log("Failed to upsert user");

    ctx.send(|m| {
        m.content(format!("Successfully linked YouTube account {}", channel_id))
            .ephemeral(true)
    }).await?;

    Ok(())
}
