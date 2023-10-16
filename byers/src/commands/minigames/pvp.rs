use std::time::Duration;

use async_trait::async_trait;
use poise::serenity_prelude::{ButtonStyle, InteractionResponseType, User};
use rand::{distributions::Standard, prelude::Distribution};

use crate::prelude::*;
use crate::{commands::minigames::Minigame, event_handlers::message::update_activity};
use judeharley::{
    communication::ByersUnixStream,
    cooldowns::{is_on_cooldown, set_cooldown, UserCooldownKey},
    db::{DbServerConfig, DbUser},
    prelude::DiscordTimestamp,
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

async fn pvp_action(ctx: ApplicationContext<'_>, user: User) -> Result<(), Error> {
    let data = ctx.data;

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let mut challenger = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;
    let mut challenged = DbUser::fetch_or_insert(&data.db, user.id.0 as i64).await?;
    let mut server_config =
        DbServerConfig::fetch_or_insert(&data.db, ctx.guild_id().unwrap().0 as i64).await?;

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

    if user.id == ctx.framework.bot_id {
        let bot_won = rand::random::<f64>() < 0.9;

        if bot_won {
            ctx.send(|m| {
                m.embed(|e| {
                    PvP::prepare_embed(e)
                        .description(format!("Byers wiped the floor with {}! They will need to rest for at least 10 minutes! Additionally, Byers took your lunch money of 10 Boondollars!", ctx.author()))
                })
            })
            .await?;
            set_cooldown(&data.redis_pool, challenger_key, 10 * 60).await?;

            challenger.boonbucks -= 10.min(challenger.boonbucks);
            server_config.slot_jackpot += 10.min(challenger.boonbucks);
        } else {
            ctx.send(|m| {
                m.embed(|e| {
                    PvP::prepare_embed(e).description(format!(
                        "Against all odds, {} came out victorious against Byers! You received Byers' collected lunch money of {} Boondollars!",
                        ctx.author(),
                        server_config.slot_jackpot,
                    ))
                })
            })
            .await?;
            set_cooldown(&data.redis_pool, challenger_key, 5 * 60).await?;

            challenged.boonbucks += server_config.slot_jackpot;
            server_config.slot_jackpot = 10;
        }

        challenger.update(&data.db).await?;
        server_config.update(&data.db).await?;

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
                    .description("You don't have enough Boondollars to challenge someone!")
            })
        })
        .await?;
        return Ok(());
    }

    if challenged.boonbucks < 10 {
        ctx.send(|m| {
            m.embed(|e| {
                PvP::prepare_embed(e).description(format!(
                    "{} doesn't have enough Boondollars to accept your challenge!",
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

    // {player2} accepted {player1}'s challenge!
    // The two warriors face each other, from opposite ends of the colosseum. The crowd roars... The wind is howling... Somewhere, a clock ticks, and the fate of our heroes hangs in the balance. FIGHT!
    // The wind picks up, consuming the colosseum in a wild sandstorm.
    // The dust settles and {player1/player2} emerges victorious!
    mci.create_interaction_response(ctx.serenity_context(), |r| {
        r.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|m| {
                m.embed(|e| {
                    PvP::prepare_embed(e).description(format!(
                        "{} accepted {}'s challenge!",
                        user.name,
                        ctx.author()
                    ))
                })
                .components(|c| c)
            })
    })
    .await?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    handle
        .edit(poise::Context::Application(ctx), |m| {
            m.embed(|e| {
                PvP::prepare_embed(e)
                    .description("The two warriors face each other, from opposite ends of the colosseum. The crowd roars... The wind is howling... Somewhere, a clock ticks, and the fate of our heroes hangs in the balance. FIGHT! Suddenly, the wind picks up, consuming the colosseum in a wild sandstorm.")
            })
        })
        .await?;

    let game = PvP;
    let result = game.play().await?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    match result {
        PvPResult::Player1 => {
            challenger.boonbucks += 10;
            challenged.boonbucks -= 10;
            challenger.update(&data.db).await?;
            challenged.update(&data.db).await?;

            handle
                .edit(poise::Context::Application(ctx), |m| {
                    m.embed(|e| {
                        PvP::prepare_embed(e).description(format!(
                            "The dust settles and {} emerges victorious!",
                            ctx.author().name,
                        ))
                    })
                })
                .await?;
        }
        PvPResult::Player2 => {
            challenged.boonbucks += 10;
            challenger.boonbucks -= 10;
            challenged.update(&data.db).await?;
            challenger.update(&data.db).await?;

            handle
                .edit(poise::Context::Application(ctx), |m| {
                    m.embed(|e| {
                        PvP::prepare_embed(e).description(format!(
                            "The dust settles and {} emerges victorious!",
                            user.name,
                        ))
                    })
                })
                .await?;
        }
    }

    set_cooldown(&data.redis_pool, challenger_key, 5 * 60).await?;
    set_cooldown(&data.redis_pool, challenged_key, 5 * 60).await?;

    Ok(())
}

/// [S] Make them pay!
#[poise::command(slash_command, guild_only)]
pub async fn pvp(
    ctx: ApplicationContext<'_>,
    #[description = "The player to challenge"] user: User,
) -> Result<(), Error> {
    pvp_action(ctx, user).await
}

/// [S] Make them pay!
#[poise::command(context_menu_command = "Minigame: PvP", guild_only)]
pub async fn pvp_context(
    ctx: ApplicationContext<'_>,
    #[description = "The player to challenge"] user: User,
) -> Result<(), Error> {
    pvp_action(ctx, user).await
}
