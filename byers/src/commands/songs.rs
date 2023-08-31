use tracing_unwrap::{OptionExt, ResultExt};

use crate::{communication::LiquidsoapCommunication, prelude::*};

pub async fn autocomplete_songs(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::AutocompleteChoice<String>> {
    let data = ctx.data();

    let songs = sqlx::query!(
        r#"
        WITH search AS (
            SELECT to_tsquery(string_agg(lexeme || ':*', ' & ' ORDER BY positions)) AS query
            FROM unnest(to_tsvector($1))
        )
        SELECT title, artist, album, file_path
        FROM songs, search
        WHERE tsvector @@ query
        "#,
        partial
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query database: {}", e);
        e
    })
    .expect_or_log("Failed to query database");

    songs
        .into_iter()
        .take(20)
        .map(|song| poise::AutocompleteChoice {
            name: format!("{} - {}", song.artist, song.title),
            value: song.file_path,
        })
}

/// Requests a song for the radio
#[poise::command(
    slash_command,
    user_cooldown = 5400,
    required_bot_permissions = "SEND_MESSAGES"
)]
pub async fn song_request(
    ctx: Context<'_>,
    #[description = "The song to request"]
    #[rest]
    #[autocomplete = "autocomplete_songs"]
    song: String,
) -> Result<(), Error> {
    let ctx = match ctx {
        Context::Application(ctx) => ctx,
        _ => unreachable!(),
    };

    let data = ctx.data();

    let song = sqlx::query!(
        r#"
        SELECT title, artist, album, file_path, duration
        FROM songs
        WHERE file_path = $1
        "#,
        song
    )
    .fetch_one(&data.db)
    .await;

    if let Err(sqlx::Error::RowNotFound) = song {
        poise::send_application_reply(ctx, |m| m.content("Song not found.").ephemeral(true))
            .await?;
        return Ok(());
    }
    let song = song?;

    // check if a song has been requested already within the following conditions:
    // - if the song is shorter than 5 minutes, check if it has been played within the last 1800 seconds
    // - if the song is 5 minutes or longer but shorter than 10 minutes, check if it has been played within the last 3600 seconds
    // - if the song is 10 minutes or longer, check if it has been played within the last 5413 seconds
    let last_played = sqlx::query!(
        r#"
        SELECT created_at
        FROM song_requests
        WHERE song_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        song.file_path
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query database: {}", e);
        e
    })?;

    let last_played = if let Some(last_played) = last_played {
        last_played.created_at
    } else {
        chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
    };

    let cooldown_time = if song.duration < 300.0 {
        std::time::Duration::from_secs(1800)
    } else if song.duration < 600.0 {
        std::time::Duration::from_secs(3600)
    } else {
        std::time::Duration::from_secs(5413)
    };

    let over = last_played
        + chrono::Duration::from_std(cooldown_time)
            .expect_or_log("Failed to convert std::time::Duration to chrono::Duration");

    if over > chrono::Utc::now().naive_utc() {
        let discord_relative = format!("<t:{}:R>", over.timestamp());
        ctx.send(|b| {
            b.content(format!(
                "This song has been requested recently. You can request this song again {}",
                discord_relative
            ))
            .ephemeral(true)
        })
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message: {}", e);
            e
        })?;
        return Ok(());
    }

    let result = {
        let mut telnet = data.comms.lock().await;
        telnet
            .request_song(&song.file_path)
            .await
            .expect_or_log("Failed to request song")
    };

    sqlx::query!(
        r#"
        INSERT INTO song_requests (song_id, user_id)
        VALUES ($1, $2)
        "#,
        song.file_path,
        ctx.author().id.0 as i64
    )
    .execute(&data.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert song request: {}", e);
        e
    })?;

    let cooldown_time = std::time::Duration::from_secs(5400);
    let over = chrono::Utc::now()
        + chrono::Duration::from_std(cooldown_time)
            .expect_or_log("Failed to convert std::time::Duration to chrono::Duration");
    let discord_relative = over.relative_time();
    ctx.send(|b| {
        b.content(format!(r#""{} - {}" requested! You can request again in 1 and 1/2 hours ({discord_relative}). (Request ID {result})"#, &song.album, &song.title))
            .ephemeral(true)
    })
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message: {}", e);
            e
        })?;

    Ok(())
}
