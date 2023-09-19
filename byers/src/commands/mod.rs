use tracing_unwrap::ResultExt;

use crate::{db::DbSong, prelude::Context};
use ellipse::Ellipse;

pub mod admin;
pub mod currency;
pub mod help;
pub mod minigames;
pub mod songs;
pub mod version;
pub mod youtube;
pub mod add_stuff;

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
