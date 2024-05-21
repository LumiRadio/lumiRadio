use std::time::Duration;

use futures::StreamExt;
use poise::serenity_prelude::{
    ButtonStyle, ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption,
};
use poise::CreateReply;

use crate::commands::{autocomplete_favourite_songs, autocomplete_songs};
use crate::event_handlers::message::update_activity;
use crate::prelude::*;
use judeharley::{
    communication::LiquidsoapCommunication,
    cooldowns::{is_on_cooldown, set_cooldown, UserCooldownKey},
    DiscordTimestamp, SongRequests, Songs, Users,
};

/// Song-related commands
#[poise::command(
    slash_command,
    subcommands(
        "request",
        "playing",
        "history",
        "queue",
        "search",
        "favourite",
        "unfavourite",
        "request_favourite"
    ),
    subcommand_required
)]
pub async fn song(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// Displays the last 10 songs played
#[poise::command(slash_command)]
pub async fn history(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let last_songs = Songs::last_10_songs(&data.db).await?;

    let description = last_songs
        .into_iter()
        .enumerate()
        .map(|(i, song)| format!("{}. {} - {}\n", i + 1, song.album, song.title))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Song History")
                .description(format!("```\n{}\n```", description)),
        ),
    )
    .await?;

    Ok(())
}

/// Marks the current song as a favourite song
#[poise::command(slash_command, ephemeral)]
pub async fn favourite(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;
    let Some(song) = Songs::last_played(&data.db).await? else {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Song Requests")
                    .description("Nothing is currently playing!"),
            ),
        )
        .await?;
        return Ok(());
    };

    user.favourite_song(&song, &data.db).await?;
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Song Requests")
                .description("Marked the currently playing song as favourite!"),
        ),
    )
    .await?;

    Ok(())
}

/// Unfavourites the specified song
#[poise::command(slash_command, ephemeral)]
pub async fn unfavourite(
    ctx: ApplicationContext<'_>,
    #[description = "The song to unfavourite"]
    #[rest]
    #[autocomplete = "autocomplete_favourite_songs"]
    song: String,
) -> Result<(), Error> {
    let data = ctx.data;
    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;

    let Some(song) = Songs::get_by_hash(&song, &data.db).await? else {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Song Requests")
                    .description("Could not find that song!"),
            ),
        )
        .await?;
        return Ok(());
    };

    user.unfavourite_song(&song, &data.db).await?;
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Song Requests")
                .description("Unmarked the currently playing song as favourite!"),
        ),
    )
    .await?;

    Ok(())
}

/// Displays the currently playing song
#[poise::command(slash_command)]
pub async fn playing(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let Some(current_song) = Songs::last_played(&data.db).await? else {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Currently Playing")
                    .description("Nothing is currently playing!"),
            ),
        )
        .await?;
        return Ok(());
    };
    let play_count = current_song.played(&data.db).await?;
    let request_count = current_song.requested(&data.db).await?;

    ctx.send(
        CreateReply::default().embed(CreateEmbed::new().title("Currently Playing").description(
            format!(
                "{} - {}\n\nThis song has been played {} times and requested {} times.",
                current_song.album, current_song.title, play_count, request_count
            ),
        )),
    )
    .await?;

    Ok(())
}

/// Displays the current queue
#[poise::command(slash_command)]
pub async fn queue(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let mut comms = data.comms.lock().await;
    let requests = comms.song_requests().await?;

    if requests.is_empty() {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Song Queue")
                    .description("There are no songs in the queue!"),
            ),
        )
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

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Song Queue")
                .description(format!("```\n{}\n```", queue)),
        ),
    )
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

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;

    let suggestions = Songs::search(&search, &data.db)
        .await?
        .into_iter()
        .take(20)
        .collect::<Vec<_>>();

    if suggestions.is_empty() {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Song Search")
                    .description("No songs were found matching your search."),
            ),
        )
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

    let user_cooldown = UserCooldownKey::new(ctx.author().id.get() as i64, "song_request");
    let has_cooldown = is_on_cooldown(&data.redis_pool, user_cooldown).await?;
    let mut song_selection = vec![];
    for song in &suggestions {
        if !song.is_on_cooldown(&data.db).await? {
            let option = CreateSelectMenuOption::new(
                format!("{} - {}", song.album, song.title),
                &song.file_hash,
            );

            song_selection.push(option);
        }
    }

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
    let reply = CreateReply::default().embed(
        CreateEmbed::new()
            .title("Song Search")
            .description(description),
    );
    let reply = if has_cooldown.is_none() {
        reply.components(vec![CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "song_request",
                CreateSelectMenuKind::String {
                    options: song_selection,
                },
            )
            .placeholder("Select a song")
            .min_values(1)
            .max_values(1),
        )])
    } else {
        reply
    };
    let handle = ctx.send(reply).await?;
    let message = handle.message().await?;
    let Some(mci) = message
        .await_component_interaction(ctx.serenity_context())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(120))
        .await
    else {
        handle
            .edit(
                poise::Context::Application(ctx),
                CreateReply::default().components(vec![]),
            )
            .await?;

        return Ok(());
    };

    let song = suggestions
        .into_iter()
        .find(|song| {
            let ComponentInteractionDataKind::StringSelect { values } = &mci.data.kind else {
                return false;
            };

            song.file_hash == values[0]
        })
        .ok_or(anyhow::anyhow!("Failed to find song"))?;

    let _ = {
        let mut comms = data.comms.lock().await;
        comms.request_song(&song.file_path).await?
    };

    song.request(&user, &data.db).await?;

    let cooldown_time = chrono::Duration::seconds(5400);
    let over = chrono::Utc::now() + cooldown_time;
    let discord_relative = over.relative_time();

    // r.kind(InteractionResponseType::UpdateMessage)
    //         .interaction_response_data(|b| {
    //             b.embed(|e| {
    //                 e.title("Song Requests")
    //                 .description(format!(r#""{} - {}" requested! You can request again in 1 and 1/2 hours ({discord_relative})."#, &song.album, &song.title))
    //             })
    //             .components(|c| c)
    //         })
    mci.create_response(
        ctx.serenity_context(),
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .embed(
                    CreateEmbed::new()
                        .title("Song Requests")
                        .description(format!(
                            "{} - {} requested! You can request again in 1 and 1/2 hours ({})",
                            &song.album, &song.title, discord_relative
                        )),
                )
                .components(vec![]),
        ),
    )
    .await?;

    set_cooldown(&data.redis_pool, user_cooldown, 90 * 60).await?;

    Ok(())
}

async fn request_song(ctx: ApplicationContext<'_>, song: String) -> Result<(), Error> {
    let data = ctx.data();

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;
    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;

    let user_cooldown = UserCooldownKey::new(ctx.author().id.get() as i64, "song_request");
    if let Some(over) = is_on_cooldown(&data.redis_pool, user_cooldown).await? {
        ctx.send(
            CreateReply::default().embed(CreateEmbed::new().title("Song Requests").description(
                format!("You can request a song again {}.", over.relative_time()),
            )),
        )
        .await?;
        return Ok(());
    }

    let song = Songs::get_by_hash(&song, &data.db).await?;

    let Some(song) = song else {
        ctx.send(
            CreateReply::default()
                .content("Song not found.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let Some(currently_playing) = Songs::last_played(&data.db).await? else {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Song Requests")
                        .description("Nothing is currently playing!"),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };
    if currently_playing.file_hash == song.file_hash {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Song Requests")
                        .description("This song is currently playing!"),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let last_played = SongRequests::get_last_requested_for_song(&song, &data.db).await?;
    let cooldown_time = if song.duration < 300.0 {
        chrono::Duration::seconds(1800)
    } else if song.duration < 600.0 {
        chrono::Duration::seconds(3600)
    } else {
        chrono::Duration::seconds(5413)
    };

    let over = last_played + cooldown_time;

    if over > chrono::Utc::now().naive_utc() {
        // b.embed(|e| {
        //     e.title("Song Requests").description(format!(
        //         "This song has been requested recently. You can request this song again {}",
        //         over.relative_time()
        //     ))
        // })
        ctx.send(
            CreateReply::default().embed(CreateEmbed::new().title("Song Requests").description(
                format!(
                    "This song has been requested recently. You can request this song again {}",
                    over.relative_time()
                ),
            )),
        )
        .await?;
        return Ok(());
    }

    let _ = {
        let mut comms = data.comms.lock().await;
        comms.request_song(&song.file_path).await?
    };

    song.request(&user, &data.db).await?;

    let cooldown_time = chrono::Duration::seconds(5400);
    let over = chrono::Utc::now() + cooldown_time;
    let discord_relative = over.relative_time();

    let handle = ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Song Requests")
                    .description(format!(
                        r#""{} - {}" requested! You can request again in 1 and 1/2 hours ({discord_relative})."#,
                        &song.album, &song.title
                    )),
            )
            .components(vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new("song_request_favourite")
                        .label("Mark as favourite")
                        .style(ButtonStyle::Primary)
                        .emoji('⭐'),
                    CreateButton::new("song_request_unfavourite")
                        .label("Unmark as favourite")
                        .style(ButtonStyle::Danger),
                ])
            ])
    )
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message: {}", e);
            e
        })?;

    set_cooldown(&data.redis_pool, user_cooldown, 90 * 60).await?;

    let message = handle.message().await?;
    while let Some(mci) = message
        .await_component_interactions(ctx.serenity_context())
        .timeout(Duration::from_secs(60))
        .stream()
        .next()
        .await
    {
        let interaction_user = Users::get_or_insert(mci.user.id.get(), &data.db).await?;

        match mci.data.custom_id.as_ref() {
            "song_request_favourite" => {
                interaction_user.favourite_song(&song, &data.db).await?;
                mci.create_response(
                    ctx.serenity_context(),
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content("Marked as favourite!"),
                    ),
                )
                .await?;
            }
            "song_request_unfavourite" => {
                interaction_user.unfavourite_song(&song, &data.db).await?;
                mci.create_response(
                    ctx.serenity_context(),
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content("Unmarked as favourite!"),
                    ),
                )
                .await?;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// Requests a song for the radio from your favourites
#[poise::command(slash_command)]
pub async fn request_favourite(
    ctx: ApplicationContext<'_>,
    #[description = "The song to request"]
    #[rest]
    #[autocomplete = "autocomplete_favourite_songs"]
    song: String,
) -> Result<(), Error> {
    request_song(ctx, song).await
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
    request_song(ctx, song).await
}
