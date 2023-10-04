use crate::commands::autocomplete_songs;
use byers::{
    communication::LiquidsoapCommunication,
    db::DbSong,
    prelude::{ApplicationContext, Error},
};

/// Reconnects the Liquidsoap command socket
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn reconnect(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    comms.reconnect().await?;
    ctx.send(|m| m.content("Reconnected to Liquidsoap")).await?;

    Ok(())
}

/// Sends a command to the Liquidsoap server
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn control_cmd(
    ctx: ApplicationContext<'_>,
    #[description = "Command to send"] command: String,
) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    let mut response = comms.send_wait(&command).await?.trim().to_string();
    response.truncate(2000);
    ctx.send(|m| {
        m.embed(|e| {
            e.title("Command Response")
                .description(format!("```\n{}\n```", response))
                .field("Command", command, false)
        })
    })
    .await?;

    Ok(())
}

/// Gets all info about a song
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn song_info(
    ctx: ApplicationContext<'_>,
    #[description = "Song to get info about"] song: String,
) -> Result<(), Error> {
    let data = ctx.data;

    let Some(song) = DbSong::fetch_from_hash(&data.db, &song).await? else {
        ctx.send(|m| m.content("Song not found.")).await?;
        return Ok(());
    };

    Ok(())
}

/// Gets or sets the volume of the radio
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn volume(
    ctx: ApplicationContext<'_>,
    #[description = "Volume to set"]
    #[min = 0]
    #[max = 100]
    volume: Option<i32>,
) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    let Some(volume) = volume else {
        let set_volume = comms.send_wait("var.get volume").await?;
        let set_volume = set_volume.trim().parse::<f32>().unwrap_or(0.0);
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Volume")
                    .description(format!("Volume is set to {}%", (set_volume * 100.0) as i32))
            })
        }).await?;
        return Ok(());
    };

    let _ = comms
        .send_wait(&format!("var.set volume {}", volume as f32 / 100.0))
        .await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Volume Set")
                .description(format!("Volume set to {}%", volume))
        })
    })
    .await?;

    Ok(())
}

/// Pauses the radio
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn pause(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    let _ = comms.send_wait("lumiradio.pause").await?;

    ctx.send(|m| m.embed(|e| e.title("Paused").description("Paused the radio")))
        .await?;

    Ok(())
}

/// Queues a song to be played immediately after the current song
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn queue(
    ctx: ApplicationContext<'_>,
    #[description = "The song to request"]
    #[rest]
    #[autocomplete = "autocomplete_songs"]
    song: String,
) -> Result<(), Error> {
    let data = ctx.data;
    let Some(song) = DbSong::fetch_from_hash(&ctx.data.db, &song).await? else {
        ctx.send(|m| m.content("Song not found."))
            .await?;
        return Ok(());
    };

    {
        let mut comms = data.comms.lock().await;
        comms.priority_request(&song.file_path).await?;
    }
    song.request(&data.db, ctx.author().id.0).await?;

    ctx.send(|m| m.embed(|e| e.title("Song Queued").description(format!("Queued {song}"))))
        .await?;

    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum SkipType {
    #[name = "The current song"]
    Radio,
    #[name = "The next user song request"]
    SongRequest,
    #[name = "The next admin song request"]
    PriorityRequest,
}

/// Skips the current song, or the next user or admin song request
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn skip(
    ctx: ApplicationContext<'_>,
    #[description = "What to skip"] skip_type: SkipType,
) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    let command = match skip_type {
        SkipType::Radio => "lumiradio.skip",
        SkipType::SongRequest => "srq.skip",
        SkipType::PriorityRequest => "prioq.skip",
    };

    let _ = comms.send_wait(command).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Skipped")
                .description(format!("Skipped {}", skip_type))
        })
    })
    .await?;

    Ok(())
}
