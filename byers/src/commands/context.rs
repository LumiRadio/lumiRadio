use judeharley::PlayedSongs;
use poise::{serenity_prelude::CreateEmbed, CreateReply};

use crate::prelude::{ApplicationContext, Error};

#[poise::command(context_menu_command = "What song played here?")]
pub async fn what_song(
    ctx: ApplicationContext<'_>,
    #[description = "The message to check"] message: poise::serenity_prelude::Message,
) -> Result<(), Error> {
    let data = ctx.data();

    let song = PlayedSongs::get_playing_at(message.timestamp.naive_utc(), &data.db).await?;
    let Some(song) = song else {
        ctx.send(
            CreateReply::default().embed(CreateEmbed::new().title("No song found").description(
                format!(
                    "No song was found playing at [that]({}) time",
                    message.link()
                ),
            )),
        )
        .await?;
        return Ok(());
    };

    // m.embed(|e| {
    //     e.title("Song found").description(format!(
    //         "The song playing at that time was **{} - {}**.",
    //         song.album, song.title
    //     ))
    // })
    ctx.send(
        CreateReply::default().embed(CreateEmbed::new().title("Song found").description(format!(
            "The song playing at [that]({}) time was **{} - {}**.",
            message.link(),
            song.album,
            song.title
        ))),
    )
    .await?;

    Ok(())
}
