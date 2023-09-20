use tracing_unwrap::{OptionExt, ResultExt};

use crate::{
    commands::autocomplete_songs,
    communication::LiquidsoapCommunication,
    cooldowns::{is_on_cooldown, set_cooldown, UserCooldownKey},
    db::DbSong,
    prelude::*,
};

/// Song-related commands
#[poise::command(
    slash_command,
    subcommands("request", "playing", "history", "queue"),
    subcommand_required
)]
pub async fn song(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// Displays the last 10 songs played
#[poise::command(slash_command)]
pub async fn history(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;
    let last_songs = DbSong::last_10_songs(&data.db).await?;

    let description = last_songs
        .into_iter()
        .enumerate()
        .map(|(i, song)| format!("{}. {} - {}\n", i + 1, song.album, song.title))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Song History")
                .description(format!("```\n{}\n```", description))
        })
    })
    .await?;

    Ok(())
}

/// Displays the currently playing song
#[poise::command(slash_command)]
pub async fn playing(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;
    let Some(current_song) = DbSong::last_played_song(&data.db).await? else {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Currently Playing")
                    .description("Nothing is currently playing!")
            })
        }).await?;
        return Ok(());
    };
    let play_count = current_song.played(&data.db).await?;
    let request_count = current_song.requested(&data.db).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Currently Playing").description(format!(
                "{} - {}\n\nThis song has been played {} times and requested {} times.",
                current_song.album, current_song.title, play_count, request_count
            ))
        })
    })
    .await?;

    Ok(())
}

/// Displays the current queue
#[poise::command(slash_command)]
pub async fn queue(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;

    let mut comms = data.comms.lock().await;
    let queue = comms
        .song_requests()
        .await?
        .into_iter()
        .enumerate()
        .map(|(i, song)| {
            format!(
                "{}. {} - {}",
                i + 1,
                song.album.unwrap_or("<no album>".to_string()),
                song.title
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Song Queue")
                .description(format!("```\n{}\n```", queue))
        })
    })
    .await?;

    Ok(())
}

/// Requests a song for the radio
#[poise::command(slash_command)]
pub async fn request(
    ctx: ApplicationContext<'_>,
    #[description = "The song to request"]
    #[rest]
    #[autocomplete = "autocomplete_songs"]
    song: String,
) -> Result<(), Error> {
    let data = ctx.data();

    let user_cooldown = UserCooldownKey::new(ctx.author().id.0 as i64, "song_request");
    if let Some(over) = is_on_cooldown(&data.redis_pool, user_cooldown).await? {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Song Requests").description(format!(
                    "You can request a song again {}.",
                    over.relative_time(),
                ))
            })
        })
        .await?;
        return Ok(());
    }

    let song = DbSong::fetch_from_hash(&data.db, &song).await?;

    let Some(song) = song else {
        ctx.send(|m| m.content("Song not found.").ephemeral(true))
            .await?;
        return Ok(());
    };

    let Some(currently_playing) = DbSong::last_played_song(&data.db).await? else {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Song Requests")
                    .description("Nothing is currently playing!")
            })
            .ephemeral(true)
        })
        .await?;
        return Ok(());
    };
    if currently_playing.file_hash == song.file_hash {
        ctx.send(|b| {
            b.embed(|e| {
                e.title("Song Requests")
                    .description("This song is currently playing!")
            })
            .ephemeral(true)
        })
        .await?;
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
        })
        .await?;
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
    })
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message: {}", e);
            e
        })?;

    set_cooldown(&data.redis_pool, user_cooldown, 90 * 60).await?;

    Ok(())
}
