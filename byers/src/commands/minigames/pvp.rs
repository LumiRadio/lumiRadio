use std::time::Duration;

use async_trait::async_trait;
use judeharley::sea_orm::Set;
use poise::serenity_prelude::{
    ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseMessage, Mentionable, User,
};
use poise::CreateReply;
use rand::{distributions::Standard, prelude::Distribution};

use crate::prelude::*;
use crate::{commands::minigames::Minigame, event_handlers::message::update_activity};
use judeharley::{
    communication::ByersUnixStream,
    cooldowns::{is_on_cooldown, set_cooldown, UserCooldownKey},
    prelude::DiscordTimestamp,
    ServerConfig, Users,
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

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let challenger = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;
    let challenged = Users::get_or_insert(user.id.get(), &data.db).await?;
    let server_config =
        ServerConfig::get_or_insert(ctx.guild_id().unwrap().get(), &data.db).await?;

    let challenger_key = UserCooldownKey::new(challenger.id, "pvp");
    let challenged_key = UserCooldownKey::new(challenged.id, "pvp");
    if let Some(over) = is_on_cooldown(&data.redis_pool, challenger_key).await? {
        ctx.send(
            CreateReply::default().embed(
                PvP::prepare_embed()
                    .description(format!(
                        "You need to rest! You can challenge someone again {}.",
                        over.relative_time(),
                    ))
                    .to_owned(),
            ),
        )
        .await?;
        return Ok(());
    }

    if user.id == ctx.framework.bot_id {
        let bot_won = rand::random::<f64>() < 0.9;

        if bot_won {
            let cost = 10.min(challenger.boonbucks);
            ctx.send(
                CreateReply::default()
                    .embed(
                        PvP::prepare_embed()
                            .description(format!(
                                "Byers wiped the floor with {}! They will need to rest for at least 10 minutes! Additionally, Byers took your lunch money of {} Boondollars!",
                                ctx.author(),
                                cost,
                            ))
                    )
            )
            .await?;
            set_cooldown(&data.redis_pool, challenger_key, 10 * 60).await?;

            let new_boonbucks = challenger.boonbucks - cost;
            let new_jackpot = server_config.slot_jackpot + cost;
            challenger
                .update(
                    judeharley::entities::users::ActiveModel {
                        boonbucks: Set(new_boonbucks as i32),
                        ..Default::default()
                    },
                    &data.db,
                )
                .await?;
            server_config
                .update(
                    judeharley::entities::server_config::ActiveModel {
                        slot_jackpot: Set(new_jackpot),
                        ..Default::default()
                    },
                    &data.db,
                )
                .await?;
        } else {
            ctx.send(
                CreateReply::default()
                    .embed(
                        PvP::prepare_embed()
                            .description(format!(
                                "Against all odds, {} came out victorious against Byers! You received Byers' collected lunch money of {} Boondollars!",
                                ctx.author(),
                                server_config.slot_jackpot
                            ))
                    )
            )
            .await?;
            set_cooldown(&data.redis_pool, challenger_key, 5 * 60).await?;

            let new_boonbucks = challenger.boonbucks + server_config.slot_jackpot;
            challenger
                .update(
                    judeharley::entities::users::ActiveModel {
                        boonbucks: Set(new_boonbucks as i32),
                        ..Default::default()
                    },
                    &data.db,
                )
                .await?;
            server_config
                .update(
                    judeharley::entities::server_config::ActiveModel {
                        slot_jackpot: Set(10),
                        ..Default::default()
                    },
                    &data.db,
                )
                .await?;
        }

        return Ok(());
    }

    if challenger.id == challenged.id {
        ctx.send(
            CreateReply::default()
                .embed(PvP::prepare_embed().description("You can't challenge yourself to a duel!")),
        )
        .await?;
        return Ok(());
    }

    if challenger.boonbucks < 10 {
        ctx.send(
            CreateReply::default().embed(
                PvP::prepare_embed()
                    .description("You don't have enough Boondollars to challenge someone!"),
            ),
        )
        .await?;
        return Ok(());
    }

    if challenged.boonbucks < 10 {
        ctx.send(
            CreateReply::default().embed(PvP::prepare_embed().description(format!(
                "{} doesn't have enough Boondollars to accept your challenge!",
                user.name
            ))),
        )
        .await?;
        return Ok(());
    }

    let handle = ctx
        .send(
            CreateReply::default()
                .embed(PvP::prepare_embed().description(format!(
                    "{} challenged {} to a duel! Do you accept?\n\nYou have 60 seconds to respond.",
                    ctx.author().mention(),
                    user
                )))
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(format!("pvp_accept_{}", challenger.id))
                        .label("Accept")
                        .style(ButtonStyle::Success)
                        .emoji('✅'),
                    CreateButton::new(format!("pvp_decline_{}", challenger.id))
                        .label("Decline")
                        .style(ButtonStyle::Danger)
                        .emoji('❌'),
                ])]),
        )
        .await?;
    let message = handle.message().await?;
    let Some(mci) = message
        .await_component_interaction(ctx.serenity_context())
        .author_id((challenged.id as u64).into())
        .channel_id(message.channel_id)
        .timeout(Duration::from_secs(60))
        .filter(move |mci| {
            mci.data.custom_id == format!("pvp_accept_{}", challenger.id)
                || mci.data.custom_id == format!("pvp_decline_{}", challenger.id)
        })
        .await
    else {
        handle
            .edit(
                poise::Context::Application(ctx),
                CreateReply::default()
                    .embed(PvP::prepare_embed().description(format!(
                        "{} challenged {} to a duel and they didn't respond in time!",
                        ctx.author().name,
                        user.name,
                    )))
                    .components(vec![]),
            )
            .await?;
        return Ok(());
    };
    if mci.data.custom_id != format!("pvp_accept_{}", challenger.id) {
        mci.create_response(
            ctx.serenity_context(),
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(PvP::prepare_embed().description(format!(
                        "{} challenged {} to a duel and they declined!",
                        ctx.author().name,
                        user.name,
                    )))
                    .components(vec![]),
            ),
        )
        .await?;
        return Ok(());
    }

    // {player2} accepted {player1}'s challenge!
    // The two warriors face each other, from opposite ends of the colosseum. The crowd roars... The wind is howling... Somewhere, a clock ticks, and the fate of our heroes hangs in the balance. FIGHT!
    // The wind picks up, consuming the colosseum in a wild sandstorm.
    // The dust settles and {player1/player2} emerges victorious!
    mci.create_response(
        ctx.serenity_context(),
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .embed(PvP::prepare_embed().description(format!(
                    "{} accepted {}'s challenge!",
                    user.name,
                    ctx.author()
                )))
                .components(vec![]),
        ),
    )
    .await?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    handle
        .edit(
            poise::Context::Application(ctx),
            CreateReply::default()
                .embed(PvP::prepare_embed().description("The two warriors face each other, from opposite ends of the colosseum. The crowd roars... The wind is howling... Somewhere, a clock ticks, and the fate of our heroes hangs in the balance. FIGHT! Suddenly, the wind picks up, consuming the colosseum in a wild sandstorm.")),
        )
        .await?;

    let game = PvP;
    let result = game.play().await?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    match result {
        PvPResult::Player1 => {
            let challenger_boonbucks = challenger.boonbucks + 10;
            let challenged_boonbucks = challenged.boonbucks - 10;

            challenger
                .update(
                    judeharley::entities::users::ActiveModel {
                        boonbucks: Set(challenger_boonbucks as i32),
                        ..Default::default()
                    },
                    &data.db,
                )
                .await?;

            challenged
                .update(
                    judeharley::entities::users::ActiveModel {
                        boonbucks: Set(challenged_boonbucks as i32),
                        ..Default::default()
                    },
                    &data.db,
                )
                .await?;

            handle
                .edit(
                    poise::Context::Application(ctx),
                    CreateReply::default().embed(PvP::prepare_embed().description(format!(
                        "The dust settles and {} emerged victorious!",
                        ctx.author().name,
                    ))),
                )
                .await?;
        }
        PvPResult::Player2 => {
            let challenger_boonbucks = challenger.boonbucks - 10;
            let challenged_boonbucks = challenged.boonbucks + 10;

            challenger
                .update(
                    judeharley::entities::users::ActiveModel {
                        boonbucks: Set(challenger_boonbucks as i32),
                        ..Default::default()
                    },
                    &data.db,
                )
                .await?;

            challenged
                .update(
                    judeharley::entities::users::ActiveModel {
                        boonbucks: Set(challenged_boonbucks as i32),
                        ..Default::default()
                    },
                    &data.db,
                )
                .await?;

            handle
                .edit(
                    poise::Context::Application(ctx),
                    CreateReply::default().embed(PvP::prepare_embed().description(format!(
                        "The dust settles and {} emerged victorious!",
                        user.name
                    ))),
                )
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
