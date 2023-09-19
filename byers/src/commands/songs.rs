use tracing_unwrap::{OptionExt, ResultExt};

use crate::{
    commands::autocomplete_songs, communication::LiquidsoapCommunication, db::DbSong, prelude::*,
};

/// Song-related commands
#[poise::command(slash_command, subcommands("request"))]
pub async fn song(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// Requests a song for the radio
#[poise::command(slash_command, ephemeral, user_cooldown = 5400)]
pub async fn request(
    ctx: ApplicationContext<'_>,
    #[description = "The song to request"]
    #[rest]
    #[autocomplete = "autocomplete_songs"]
    song: String,
) -> Result<(), Error> {
    let data = ctx.data();

    let song = DbSong::fetch_from_hash(&data.db, &song).await?;

    let Some(song) = song else {
        ctx.send(|m| m.content("Song not found.").ephemeral(true))
            .await?;
        return Ok(());
    };

    let currently_playing = DbSong::last_played_song(&data.db).await?;
    if currently_playing.file_hash == song.file_hash {
        ctx.send(|b| {
            b.embed(|e| {
                e.title("Song Requests")
                    .description("This song is currently playing!")
            })
            .ephemeral(true)
        })
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message: {}", e);
            e
        })?;
        return Ok(());
    }

    let last_played = song.last_requested(&data.db).await?;
    let cooldown_time = if song.duration < 300.0 {
        chrono::Duration::seconds(1800)
    } else if song.duration < 600.0 {
        chrono::Duration::seconds(3600)
    } else {
        chrono::Duration::seconds(5413)
    };

    let over = last_played + cooldown_time;

    if over > chrono::Utc::now().naive_utc() {
        ctx.send(|b| {
            b.embed(|e| {
                e.title("Song Requests").description(format!(
                    "This song has been requested recently. You can request this song again {}",
                    over.relative_time()
                ))
            })
            .ephemeral(true)
        })
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message: {}", e);
            e
        })?;
        return Ok(());
    }

    let _ = {
        let mut comms = data.comms.lock().await;
        comms
            .request_song(&song.file_path)
            .await
            .expect_or_log("Failed to request song")
    };

    song.request(&data.db, ctx.author().id.0).await?;

    let cooldown_time = chrono::Duration::seconds(5400);
    let over = chrono::Utc::now() + cooldown_time;
    let discord_relative = over.relative_time();
    ctx.send(|b| {
        b.embed(|e| {
            e.title("Song Requests")
            .description(format!(r#""{} - {}" requested! You can request again in 1 and 1/2 hours ({discord_relative})."#, &song.album, &song.title))
        })
        .ephemeral(true)
    })
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message: {}", e);
            e
        })?;

    Ok(())
}
