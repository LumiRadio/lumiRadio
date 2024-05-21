use poise::serenity_prelude::{AutocompleteChoice, CreateActionRow, CreateButton, CreateEmbed};
use poise::CreateReply;
use tracing_unwrap::ResultExt;

use crate::event_handlers::message::update_activity;
use crate::prelude::*;
use ellipse::Ellipse;
use judeharley::{Songs, Users};

pub mod add_stuff;
pub mod admin;
pub mod context;
pub mod currency;
pub mod help;
pub mod minigames;
pub mod songs;
pub mod version;
pub mod youtube;

/// Displays the link to the radio
#[poise::command(slash_command)]
pub async fn listen(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    update_activity(ctx.data, ctx.author().id, ctx.channel_id()).await?;

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("lumiRadio is now playing")
                    .description(
                        "Add the following link to your favourite radio player: https://listen.lumirad.io/",
                    ),
            )
            .components(vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new_link("https://listen.lumirad.io/")
                        .label("Listen")
                        .emoji('🔗'),
                ])
            ])
    )
    .await?;

    Ok(())
}

pub async fn autocomplete_songs(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    let data = ctx.data();

    let songs = Songs::search(partial, &data.db)
        .await
        .expect_or_log("Failed to query database");

    songs.into_iter().take(20).map(|song| {
        AutocompleteChoice::new(
            format!("{} - {}", song.artist, song.title)
                .as_str()
                .truncate_ellipse(97),
            song.file_hash,
        )
    })
}

pub async fn autocomplete_favourite_songs(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    let data = ctx.data();
    let user = Users::get_or_insert(ctx.author().id.get(), &data.db)
        .await
        .expect_or_log("Failed to query database");

    let songs = Songs::search_favourited_songs(partial, &user, &data.db)
        .await
        .expect_or_log("Failed to query database");

    songs.into_iter().take(20).map(|song| {
        AutocompleteChoice::new(
            format!("{} - {}", song.artist, song.title)
                .as_str()
                .truncate_ellipse(97),
            song.file_hash,
        )
    })
}
