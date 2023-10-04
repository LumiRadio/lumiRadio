use std::time::Duration;

use poise::serenity_prelude::{CreateSelectMenuOption, InteractionResponseType};
use tracing_unwrap::{OptionExt, ResultExt};

use crate::commands::autocomplete_songs;
use crate::event_handlers::message::update_activity;
use byers::{
    communication::LiquidsoapCommunication,
    cooldowns::{is_on_cooldown, set_cooldown, UserCooldownKey},
    db::DbSong,
    prelude::*,
};

/// Song-related commands
#[poise::command(
    slash_command,
    subcommands("request", "playing", "history", "queue", "search"),
    subcommand_required
)]
pub async fn song(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// Displays the last 10 songs played
#[poise::command(slash_command)]
pub async fn history(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

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

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

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

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let mut comms = data.comms.lock().await;
    let requests = comms.song_requests().await?;

    if requests.is_empty() {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Song Queue")
                    .description("There are no songs in the queue!")
            })
        })
        .await?;
        return Ok(());
    }

    let queue = requests
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

/// Lets you search for a song and then request it
#[poise::command(slash_command)]
pub async fn search(
    ctx: ApplicationContext<'_>,
    #[description = "The song to search for"] search: String,
) -> Result<(), Error> {
    let data = ctx.data;

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let suggestions = DbSong::search(&data.db, &search)
        .await?
        .into_iter()
        .take(20)
        .collect::<Vec<_>>();

    if suggestions.is_empty() {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Song Search")
                    .description("No songs were found matching your search.")
            })
        })
        .await?;
        return Ok(());
    }

    let suggestion_str = suggestions
        .iter()
        .enumerate()
        .map(|(i, song)| format!("{}. {} - {}", i + 1, song.album, song.title))
        .collect::<Vec<_>>()
        .join("\n");
    let results = suggestions.len();

    let user_cooldown = UserCooldownKey::new(ctx.author().id.0 as i64, "song_request");
    let has_cooldown = is_on_cooldown(&data.redis_pool, user_cooldown).await?;
    let mut song_selection = vec![];
    for song in &suggestions {
        if !song.is_on_cooldown(&data.db).await? {
            let mut option = CreateSelectMenuOption::default();
            option.label(format!("{} - {}", song.album, song.title));
            option.value(song.file_hash.clone());

            song_selection.push(option);
        }
    }

    let handle = ctx.send(|m| {
        let reply = m.embed(|e| {
            let mut description = format!(
                "Here are the top {results} results for your search for `{search}`.\n\n```\n{suggestion_str}\n```"
            );
            if let Some(over) = has_cooldown.as_ref() {
                description.push_str(&format!(
                    "\n\nYou can request a song again {}.",
                    over.relative_time()
                ));
            } else {
                description.push_str("\n\nYou may request one of them now by selecting them below within 2 minutes. Songs that are currently on cooldown will not be selectable.");
            }

            e.title("Song Search").description(description)
        });

        if has_cooldown.is_none() {
            reply.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_select_menu(|sm| {
                        sm.custom_id("song_request")
                            .placeholder("Select a song")
                            .min_values(1)
                            .max_values(1)
                            .options(|o| {
                                o.set_options(song_selection)
                            })
                    })
                })
            });
        }

        reply
    })
    .await?;
    let message = handle.message().await?;
    let Some(mci) = message
        .await_component_interaction(ctx.serenity_context())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(120))
        .await else {
            handle.edit(poise::Context::Application(ctx), |m| {
                m.components(|c| c)
            }).await?;

            return Ok(());
        };

    let song = suggestions
        .into_iter()
        .find(|song| song.file_hash == mci.data.values[0])
        .expect_or_log("Failed to find song");

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

    mci.create_interaction_response(ctx.serenity_context(), |r| {
        r.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|b| {
                b.embed(|e| {
                    e.title("Song Requests")
                    .description(format!(r#""{} - {}" requested! You can request again in 1 and 1/2 hours ({discord_relative})."#, &song.album, &song.title))
                })
                .components(|c| c)
            })
    }).await?;

    set_cooldown(&data.redis_pool, user_cooldown, 90 * 60).await?;

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

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

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
