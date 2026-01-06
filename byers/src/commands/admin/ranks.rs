use judeharley::SlcbRank;
use poise::serenity_prelude::{AutocompleteChoice, User};

use crate::prelude::{ApplicationContext, Context, Error};

#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    subcommands("add", "remove", "update"),
    subcommand_required
)]
pub async fn ranks(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

pub async fn autocomplete_ranks(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    let data = ctx.data();
    let ranks = SlcbRank::get_all(&data.db).await.unwrap();
    
    ranks.into_iter().take(20)
        .filter(|rank| rank.rank_name.to_lowercase().contains(&partial.to_lowercase()))
        .map(|rank| {
            AutocompleteChoice::new(rank.rank_name, rank.id)
        })
}

#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn add(
    ctx: ApplicationContext<'_>,
    #[description = "The rank name"]
    rank_name: String,
    #[description = "The hour requirement"]
    hour_requirement: i32,
    #[description = "The user this rank is for"]
    user_id: Option<User>,
) -> Result<(), Error> {
    let data = ctx.data();
    SlcbRank::create(
        &rank_name,
        hour_requirement,
        None,
        user_id.map(|u| u.id.get() as i64),
        &data.db,
    ).await?;

    ctx.say("Rank was successfully added").await?;

    Ok(())
}

#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn remove(
    ctx: ApplicationContext<'_>,
    #[description = "The rank to remove"]
    #[autocomplete = "autocomplete_ranks"]
    rank: i32,
) -> Result<(), Error> {
    let data = ctx.data();
    SlcbRank::delete(rank, &data.db).await?;

    ctx.say("Rank removed").await?;

    Ok(())
}

#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn update(
    ctx: ApplicationContext<'_>,
    #[description = "The rank to update"]
    #[autocomplete = "autocomplete_ranks"]
    rank: i32,
    #[description = "The new rank name"]
    rank_name: Option<String>,
    #[description = "The new hour requirement"]
    hour_requirement: Option<i32>,
    #[description = "The user this rank is for"]
    user_id: Option<User>,
) -> Result<(), Error> {
    let data = ctx.data();
    SlcbRank::update(
        rank,
        rank_name.as_deref(),
        hour_requirement,
        None,
        user_id.map(|u| u.id.get() as i64),
        &data.db,
    ).await?;

    ctx.say("Rank was successfully updated").await?;

    Ok(())
}

