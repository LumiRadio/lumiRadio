use tracing_unwrap::ResultExt;

use crate::{db::DbSong, prelude::Context};
use ellipse::Ellipse;
use crate::prelude::{ApplicationContext, Error};

pub mod admin;
pub mod currency;
pub mod help;
pub mod minigames;
pub mod songs;
pub mod version;
pub mod youtube;
pub mod add_stuff;

#[poise::command(slash_command)]
pub async fn listen(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    ctx.send(|m| {
        m.embed(|e| {
            e.title("lumiRadio is now playing")
                .description("Add the following link to your favourite radio player: https://listen.lumirad.io/")
        })
            .components(|c| {
                c.create_action_row(|ar| {
                    ar.create_button(|b| {
                        b.label("Listen")
                            .style(poise::serenity_prelude::ButtonStyle::Link)
                            .emoji('🔗')
                            .url("https://listen.lumirad.io/")
                    })
                })
            })
    }).await?;

    Ok(())
}

pub async fn autocomplete_songs(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::AutocompleteChoice<String>> {
    let data = ctx.data();

    let songs = DbSong::search(&data.db, partial)
        .await
        .expect_or_log("Failed to query database");

    songs
        .into_iter()
        .take(20)
        .map(|song| poise::AutocompleteChoice {
            name: format!("{} - {}", song.artist, song.title)
                .as_str()
                .truncate_ellipse(97)
                .to_string(),
            value: song.file_hash,
        })
}
