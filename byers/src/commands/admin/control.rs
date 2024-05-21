use std::path::PathBuf;

use crate::commands::autocomplete_songs;
use crate::prelude::*;
use judeharley::{communication::LiquidsoapCommunication, Songs, Users};
use poise::{serenity_prelude::CreateEmbed, CreateReply};

/// Reconnects the Liquidsoap command socket
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn reconnect(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    comms.reconnect().await?;
    ctx.send(CreateReply::default().content("Reconnected to Liquidsoap"))
        .await?;

    Ok(())
}

/// Reindexes the song database
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn reindex(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;
    let mut comms = ctx.data.comms.lock().await;

    ctx.defer_ephemeral().await?;
    judeharley::maintenance::indexing::index(&data.db, "/music".into()).await?;
    let playlist_path = PathBuf::from("/music/playlist.m3u");
    judeharley::maintenance::indexing::create_playlist(&data.db, &playlist_path).await?;
    comms.send_wait("playlist.m3u.reload").await?;
    ctx.send(
        CreateReply::default().content("Reindexed the song database and reloaded the playlist."),
    )
    .await?;

    Ok(())
}

/// Generates a playlist file from the database
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn generate_playlist(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;
    let mut comms = ctx.data.comms.lock().await;

    let playlist_path = PathBuf::from("/music/playlist.m3u");
    judeharley::maintenance::indexing::create_playlist(&data.db, &playlist_path).await?;

    comms.send_wait("playlist.m3u.reload").await?;
    ctx.send(
        CreateReply::default().content(
            "Regenerated the playlist. It should automatically be loaded into Liquidsoap!",
        ),
    )
    .await?;

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
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Command Response")
                .description(format!("```\n{}\n```", response))
                .field("Command", command, false),
        ),
    )
    .await?;

    Ok(())
}

/// Gets all info about a song
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn song_info(
    ctx: ApplicationContext<'_>,
    #[description = "Song to get info about"]
    #[rest]
    #[autocomplete = "autocomplete_songs"]
    song: String,
) -> Result<(), Error> {
    let data = ctx.data;

    let Some(song) = Songs::get_by_hash(&song, &data.db).await? else {
        ctx.send(CreateReply::default().content("Song not found."))
            .await?;
        return Ok(());
    };

    let tags = song.tags(&data.db).await?;
    let tags_str = tags
        .into_iter()
        .map(|t| t.tag)
        .collect::<Vec<_>>()
        .join(", ");
    // take 1024 characters or, if longer, 1021 characters and add ...
    let tags_str = if tags_str.len() > 1024 {
        format!("{}...", &tags_str[..1021])
    } else {
        tags_str
    };

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Song Info")
                    .description(format!(
                        "The song {} - {} has the following information:",
                        &song.artist, &song.title
                    ))
                    .field("Title", &song.title, true)
                    .field("Artist", &song.artist, true)
                    .field("Album", &song.album, true)
                    .field("Bitrate", song.bitrate.to_string(), true)
                    .field("File Path", &song.file_path, true)
                    .field("ID", &song.file_hash, true)
                    .field("Tags", &tags_str, true),
            )
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Queries a tag on the specific song
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn song_tag(
    ctx: ApplicationContext<'_>,
    #[description = "Song to get info about"]
    #[autocomplete = "autocomplete_songs"]
    song: String,
    #[description = "Tag to query"] tag: String,
) -> Result<(), Error> {
    let data = ctx.data;

    let Some(song) = Songs::get_by_hash(&song, &data.db).await? else {
        ctx.send(CreateReply::default().content("Song not found."))
            .await?;
        return Ok(());
    };

    let tag_value = song.tag(&tag, &data.db).await?;
    let Some(tag_value) = tag_value else {
        ctx.send(CreateReply::default().content("Tag not found."))
            .await?;
        return Ok(());
    };

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Song Info")
                    .description(format!(
                        "The song {} - {} has the following information:",
                        &song.artist, &song.title
                    ))
                    .field("Tag", &tag, true)
                    .field("Value", &tag_value, true),
            )
            .ephemeral(true),
    )
    .await?;

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
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Volume")
                    .description(format!("Volume is set to {}%", (set_volume * 100.0) as i32)),
            ),
        )
        .await?;
        return Ok(());
    };

    let _ = comms
        .send_wait(&format!("var.set volume {}", volume as f32 / 100.0))
        .await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Volume Set")
                .description(format!("Volume set to {}%", volume)),
        ),
    )
    .await?;

    Ok(())
}

/// Pauses the radio
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn pause(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    let _ = comms.send_wait("lumiradio.pause").await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Radio Paused")
                .description("The radio has been paused"),
        ),
    )
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
    let Some(song) = Songs::get_by_hash(&song, &ctx.data.db).await? else {
        ctx.send(CreateReply::default().content("Song not found."))
            .await?;
        return Ok(());
    };
    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;

    {
        let mut comms = data.comms.lock().await;
        comms.priority_request(&song.file_path).await?;
    }
    song.request(&user, &data.db).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Song Queued")
                .description(format!("Queued {} - {}", &song.artist, &song.title)),
        ),
    )
    .await?;

    Ok(())
}

#[derive(Debug, poise::ChoiceParameter, strum::Display)]
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

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Skipped")
                .description(format!("Skipped {}", skip_type)),
        ),
    )
    .await?;

    Ok(())
}
