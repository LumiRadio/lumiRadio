use std::time::Duration;

use async_trait::async_trait;
use poise::serenity_prelude::{ButtonStyle, InteractionResponseType, Member, User};
use rand::{distributions::Standard, prelude::Distribution};

use crate::{
    commands::minigames::Minigame,
    communication::ByersUnixStream,
    cooldowns::{is_on_cooldown, set_cooldown, UserCooldownKey},
    db::DbUser,
    prelude::{ApplicationContext, Data, DiscordTimestamp, Error},
};

pub enum PvPResult {
    Player1,
    Player2,
}

impl Distribution<PvPResult> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PvPResult {
        match rng.gen_range(0..=1) {
            0 => PvPResult::Player1,
            1 => PvPResult::Player2,
            _ => unreachable!(),
        }
    }
}

pub struct PvP;

#[async_trait]
impl Minigame for PvP {
    const NAME: &'static str = "PvP";
    type MinigameResult = PvPResult;

    async fn play(&self) -> Result<PvPResult, Error> {
        Ok(rand::random())
    }

    fn command() -> Vec<poise::Command<Data<ByersUnixStream>, anyhow::Error>> {
        vec![pvp()]
    }
}

/// [S] Make them pay!
#[poise::command(slash_command, context_menu_command = "Minigame: PvP")]
pub async fn pvp(
    ctx: ApplicationContext<'_>,
    #[description = "The player to challenge"] user: User,
) -> Result<(), Error> {
    let data = ctx.data;
    let mut challenger = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;
    let mut challenged = DbUser::fetch_or_insert(&data.db, user.id.0 as i64).await?;

    let challenger_key = UserCooldownKey::new(challenger.id, "pvp");
    let challenged_key = UserCooldownKey::new(challenged.id, "pvp");
    if let Some(over) = is_on_cooldown(&data.redis_pool, challenger_key).await? {
        ctx.send(|m| {
            m.embed(|e| {
                PvP::prepare_embed(e).description(format!(
                    "You need to rest! You can challenge someone again {}.",
                    over.relative_time(),
                ))
            })
        })
        .await?;
        return Ok(());
    }

    if challenger.id == challenged.id {
        ctx.send(|m| {
            m.embed(|e| {
                PvP::prepare_embed(e).description("You can't challenge yourself to a duel!")
            })
        })
        .await?;
        return Ok(());
    }

    if challenger.boonbucks < 10 {
        ctx.send(|m| {
            m.embed(|e| {
                PvP::prepare_embed(e)
                    .description("You don't have enough boonbucks to challenge someone!")
            })
        })
        .await?;
        return Ok(());
    }

    if challenged.boonbucks < 10 {
        ctx.send(|m| {
            m.embed(|e| {
                PvP::prepare_embed(e).description(format!(
                    "{} doesn't have enough boonbucks to accept your challenge!",
                    user.name,
                ))
            })
        })
        .await?;
        return Ok(());
    }

    let handle = ctx
        .send(|m| {
            m.embed(|e| {
                PvP::prepare_embed(e).description(format!(
                    "{} challenged {} to a duel! Do you accept?\n\nYou have 60 seconds to respond.",
                    ctx.author(),
                    user,
                ))
            })
            .components(|c| {
                c.create_action_row(|r| {
                    r.create_button(|b| {
                        b.label("Accept")
                            .style(ButtonStyle::Success)
                            .emoji('✅')
                            .custom_id(format!("pvp_accept_{}", challenger.id))
                    })
                    .create_button(|b| {
                        b.label("Decline")
                            .style(ButtonStyle::Danger)
                            .emoji('❌')
                            .custom_id(format!("pvp_decline_{}", challenger.id))
                    })
                })
            })
        })
        .await?;
    let message = handle.message().await?;
    let Some(mci) = message
        .await_component_interaction(ctx.serenity_context())
        .author_id(challenged.id as u64)
        .channel_id(message.channel_id)
        .timeout(Duration::from_secs(60))
        .filter(move |mci| {
            mci.data.custom_id == format!("pvp_accept_{}", challenger.id)
                || mci.data.custom_id == format!("pvp_decline_{}", challenger.id)
        })
        .await else {
        handle
            .edit(poise::Context::Application(ctx), |r| {
                r.embed(|e| {
                    PvP::prepare_embed(e).description(format!(
                        "{} challenged {} to a duel and they didn't respond in time!",
                        ctx.author().name,
                        user.name,
                    ))
                })
                .components(|c| c)
            })
            .await?;
        return Ok(());
    };
    if mci.data.custom_id != format!("pvp_accept_{}", challenger.id) {
        mci.create_interaction_response(ctx.serenity_context(), |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.embed(|e| {
                        PvP::prepare_embed(e).description(format!(
                            "{} challenged {} to a duel and they declined!",
                            ctx.author().name,
                            user.name,
                        ))
                    })
                    .components(|c| c)
                })
        })
        .await?;
        return Ok(());
    }

    let game = PvP;
    let result = game.play().await?;

    match result {
        PvPResult::Player1 => {
            challenger.boonbucks += 10;
            challenged.boonbucks -= 10;
            challenger.update(&data.db).await?;
            challenged.update(&data.db).await?;

            mci.create_interaction_response(ctx.serenity_context(), |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.embed(|e| {
                            PvP::prepare_embed(e).description(format!(
                                "{} challenged {} to a duel and won!",
                                ctx.author().name,
                                user.name,
                            ))
                        })
                        .components(|c| c)
                    })
            })
            .await?;
        }
        PvPResult::Player2 => {
            challenged.boonbucks += 10;
            challenger.boonbucks -= 10;
            challenged.update(&data.db).await?;
            challenger.update(&data.db).await?;
            mci.create_interaction_response(ctx.serenity_context(), |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.embed(|e| {
                            PvP::prepare_embed(e).description(format!(
                                "{} challenged {} to a duel and lost!",
                                ctx.author().name,
                                user.name,
                            ))
                        })
                        .components(|c| c)
                    })
            })
            .await?;
        }
    }

    set_cooldown(&data.redis_pool, challenger_key, 5 * 60).await?;
    set_cooldown(&data.redis_pool, challenged_key, 5 * 60).await?;

    Ok(())
}
