use std::{collections::HashMap, fmt::Display, time::Duration};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use poise::serenity_prelude::{InteractionResponseType, Member};
use rand::{distributions::Standard, prelude::Distribution, seq::IteratorRandom, Rng};
use sqlx::PgPool;
use tokio_stream::StreamExt;
use tracing::info;

use crate::{commands::minigames::Minigame, event_handlers::message::update_activity};
use byers::cooldowns::{is_on_cooldown, set_cooldown, GlobalCooldownKey};
use byers::prelude::Context;
use byers::{
    communication::ByersUnixStream,
    db::DbUser,
    prelude::{ApplicationContext, Data, DiscordTimestamp, Error},
};

static STRIFE_ENEMIES_BY_PLAYER_COUNT: Lazy<HashMap<i32, StrifeEnemyType>> = Lazy::new(|| {
    vec![
        (2, StrifeEnemyType::TrainingDummy),
        (3, StrifeEnemyType::Titachnid),
        (4, StrifeEnemyType::Imp),
        (5, StrifeEnemyType::Lich),
        (6, StrifeEnemyType::Basilisk),
        (7, StrifeEnemyType::Ogre),
        (8, StrifeEnemyType::Giclops),
        (9, StrifeEnemyType::Acheron),
        (10, StrifeEnemyType::LichQueen),
    ]
    .into_iter()
    .collect()
});

trait Loot {
    fn loot(&self) -> i32;
}

#[derive(Debug, Clone, Copy)]
enum StrifeEnemyType {
    TrainingDummy,
    Titachnid,
    Imp,
    Lich,
    Basilisk,
    Ogre,
    Giclops,
    Acheron,
    LichQueen,
}

impl Display for StrifeEnemyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrifeEnemyType::TrainingDummy => write!(f, "Training Dummy"),
            StrifeEnemyType::Titachnid => write!(f, "Titachnid"),
            StrifeEnemyType::Imp => write!(f, "Imp"),
            StrifeEnemyType::Lich => write!(f, "Lich"),
            StrifeEnemyType::Basilisk => write!(f, "Basilisk"),
            StrifeEnemyType::Ogre => write!(f, "Ogre"),
            StrifeEnemyType::Giclops => write!(f, "Giclops"),
            StrifeEnemyType::Acheron => write!(f, "Acheron"),
            StrifeEnemyType::LichQueen => write!(f, "Lich Queen"),
        }
    }
}

impl Loot for StrifeEnemyType {
    fn loot(&self) -> i32 {
        match *self {
            StrifeEnemyType::TrainingDummy => 10,
            StrifeEnemyType::Titachnid => 15,
            StrifeEnemyType::Imp => 20,
            StrifeEnemyType::Lich => 25,
            StrifeEnemyType::Basilisk => 30,
            StrifeEnemyType::Ogre => 35,
            StrifeEnemyType::Giclops => 40,
            StrifeEnemyType::Acheron => 45,
            StrifeEnemyType::LichQueen => 50,
        }
    }
}

#[derive(Debug, Clone, Copy, strum::Display)]
enum EnemyGristVariant {
    Amber,
    Amethyst,
    Artifact,
    Caulk,
    Chalk,
    Cobalt,
    Diamond,
    Garnet,
    Gold,
    Iodine,
    Marble,
    Mercury,
    Quartz,
    Ruby,
    Rust,
    Shale,
    Sulfur,
    Tar,
    Uranium,
    Zillium,
}

impl Distribution<EnemyGristVariant> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> EnemyGristVariant {
        match rng.gen_range(0..=19) {
            0 => EnemyGristVariant::Amber,
            1 => EnemyGristVariant::Amethyst,
            2 => EnemyGristVariant::Artifact,
            3 => EnemyGristVariant::Caulk,
            4 => EnemyGristVariant::Chalk,
            5 => EnemyGristVariant::Cobalt,
            6 => EnemyGristVariant::Diamond,
            7 => EnemyGristVariant::Garnet,
            8 => EnemyGristVariant::Gold,
            9 => EnemyGristVariant::Iodine,
            10 => EnemyGristVariant::Marble,
            11 => EnemyGristVariant::Mercury,
            12 => EnemyGristVariant::Quartz,
            13 => EnemyGristVariant::Ruby,
            14 => EnemyGristVariant::Rust,
            15 => EnemyGristVariant::Shale,
            16 => EnemyGristVariant::Sulfur,
            17 => EnemyGristVariant::Tar,
            18 => EnemyGristVariant::Uranium,
            19 => EnemyGristVariant::Zillium,
            _ => unreachable!(),
        }
    }
}

pub struct Strife {
    players: Vec<Member>,
    enemy_type: StrifeEnemyType,
    enemy_variant: EnemyGristVariant,
}

impl Strife {
    pub fn new(players: Vec<Member>) -> Option<Self> {
        if players.len() < 2 {
            return None;
        }

        let enemy_type = *STRIFE_ENEMIES_BY_PLAYER_COUNT
            .get(&(players.len() as i32).min(10))
            .unwrap();
        let enemy_variant = rand::random();

        Some(Self {
            players,
            enemy_type,
            enemy_variant,
        })
    }

    pub fn base_pot(&self) -> i32 {
        self.players.len() as i32 * 50
    }

    pub fn enemy_name(&self) -> String {
        format!("{} {}", self.enemy_variant, self.enemy_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrifeResult {
    Wipeout,
    WinSingle,
    WinHalf,
    WinFull,
}

impl Distribution<StrifeResult> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> StrifeResult {
        match rng.gen_range(0..=3) {
            0 => StrifeResult::Wipeout,
            1 => StrifeResult::WinSingle,
            2 => StrifeResult::WinHalf,
            3 => StrifeResult::WinFull,
            _ => unreachable!(),
        }
    }
}

pub struct StrifeLoot {
    boonbucks_per_player: Option<i32>,
    grist_per_player: Option<i32>,
    grist_type: Option<EnemyGristVariant>,
    winners: Vec<Member>,
    result: StrifeResult,
}

#[async_trait]
impl Minigame for Strife {
    const NAME: &'static str = "Strife";
    type MinigameResult = StrifeLoot;

    async fn play(&self) -> Result<StrifeLoot, Error> {
        let result: StrifeResult = rand::random();

        match result {
            StrifeResult::Wipeout => Ok(StrifeLoot {
                boonbucks_per_player: None,
                grist_per_player: None,
                grist_type: None,
                result,
                winners: vec![],
            }),
            StrifeResult::WinSingle => {
                let additional_loot = self.enemy_type.loot();
                Ok(StrifeLoot {
                    boonbucks_per_player: Some(self.base_pot() + additional_loot),
                    grist_per_player: Some(rand::thread_rng().gen_range(1..=10)),
                    grist_type: Some(self.enemy_variant),
                    result,
                    winners: self
                        .players
                        .clone()
                        .into_iter()
                        .choose_multiple(&mut rand::thread_rng(), 1),
                })
            }
            StrifeResult::WinHalf => {
                let additional_loot = self.enemy_type.loot();
                Ok(StrifeLoot {
                    boonbucks_per_player: Some(
                        self.base_pot() / (self.players.len() as i32 / 2) + additional_loot,
                    ),
                    grist_per_player: Some(rand::thread_rng().gen_range(1..=10)),
                    grist_type: Some(self.enemy_variant),
                    result,
                    winners: self
                        .players
                        .clone()
                        .into_iter()
                        .choose_multiple(&mut rand::thread_rng(), self.players.len() / 2),
                })
            }
            StrifeResult::WinFull => {
                let additional_loot = self.enemy_type.loot();
                Ok(StrifeLoot {
                    boonbucks_per_player: Some(
                        self.base_pot() / (self.players.len() as i32) + additional_loot,
                    ),
                    grist_per_player: Some(rand::thread_rng().gen_range(1..=10)),
                    grist_type: Some(self.enemy_variant),
                    result,
                    winners: self.players.clone(),
                })
            }
        }
    }

    fn command() -> Vec<poise::Command<Data<ByersUnixStream>, anyhow::Error>> {
        vec![strife()]
    }
}

async fn payout_multiple(
    db: &PgPool,
    players: &[Member],
    result: &StrifeLoot,
) -> Result<(), Error> {
    for winner in players {
        let mut winner_user = DbUser::fetch_or_insert(db, winner.user.id.0 as i64).await?;
        winner_user.boonbucks += result.boonbucks_per_player.unwrap();

        match result.grist_type.unwrap() {
            EnemyGristVariant::Amber => winner_user.amber += result.grist_per_player.unwrap(),
            EnemyGristVariant::Amethyst => winner_user.amethyst += result.grist_per_player.unwrap(),
            EnemyGristVariant::Artifact => winner_user.artifact += result.grist_per_player.unwrap(),
            EnemyGristVariant::Caulk => winner_user.caulk += result.grist_per_player.unwrap(),
            EnemyGristVariant::Chalk => winner_user.chalk += result.grist_per_player.unwrap(),
            EnemyGristVariant::Cobalt => winner_user.cobalt += result.grist_per_player.unwrap(),
            EnemyGristVariant::Diamond => winner_user.diamond += result.grist_per_player.unwrap(),
            EnemyGristVariant::Garnet => winner_user.garnet += result.grist_per_player.unwrap(),
            EnemyGristVariant::Gold => winner_user.gold += result.grist_per_player.unwrap(),
            EnemyGristVariant::Iodine => winner_user.iodine += result.grist_per_player.unwrap(),
            EnemyGristVariant::Marble => winner_user.marble += result.grist_per_player.unwrap(),
            EnemyGristVariant::Mercury => winner_user.mercury += result.grist_per_player.unwrap(),
            EnemyGristVariant::Quartz => winner_user.quartz += result.grist_per_player.unwrap(),
            EnemyGristVariant::Ruby => winner_user.ruby += result.grist_per_player.unwrap(),
            EnemyGristVariant::Rust => winner_user.rust += result.grist_per_player.unwrap(),
            EnemyGristVariant::Shale => winner_user.shale += result.grist_per_player.unwrap(),
            EnemyGristVariant::Sulfur => winner_user.sulfur += result.grist_per_player.unwrap(),
            EnemyGristVariant::Tar => winner_user.tar += result.grist_per_player.unwrap(),
            EnemyGristVariant::Uranium => winner_user.uranium += result.grist_per_player.unwrap(),
            EnemyGristVariant::Zillium => winner_user.zillium += result.grist_per_player.unwrap(),
        }
        winner_user.update(db).await?;
    }

    Ok(())
}

/// Join a co-op fight against a monster and win boonbucks and grist!
#[poise::command(slash_command)]
pub async fn strife(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data;

    ctx.send(|m| {
        m.embed(|e| {
            Strife::prepare_embed(e)
                .description("Strife is currently being rewritten. It will be back soon!")
        })
    })
    .await?;

    return Ok(());

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let cooldown = GlobalCooldownKey::new("strife");
    if let Some(over) = is_on_cooldown(&data.redis_pool, cooldown).await? {
        ctx.send(|m| {
            m.embed(|e| {
                Strife::prepare_embed(e).description(format!(
                    "The arena is being cleaned up! Come back {}.",
                    over.relative_time(),
                ))
            })
        })
        .await?;
        return Ok(());
    }

    info!("Strife interaction ID: {}", ctx.interaction.id().0);

    let mut user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;

    if user.boonbucks < 50 {
        ctx.send(|e| {
            e.embed(|e| {
                Strife::prepare_embed(e).description(
                    "You don't have enough Boondollars to start a strife! (50 Boondollars required)",
                )
            })
        })
        .await?;
        return Ok(());
    }
    user.boonbucks -= 50;
    user.update(&data.db).await?;
    set_cooldown(&data.redis_pool, cooldown, 5 * 60).await?;

    let now_in_2_minutes = chrono::Utc::now() + chrono::Duration::minutes(2);
    let handle = ctx.send(|m| {
        m.embed(|e| {
            Strife::prepare_embed(e)
                .description(format!("{} gathers up a crew to **STRIFE**!\n\nPress the button below to join the crew! (50 Boondollars to enter) There are currently {} waiting to fight!\n\nThe battle should start {}", ctx.author(), 1, now_in_2_minutes.relative_time()))
        })
        .components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id("strife_join")
                        .label("Join")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                })
            })
        })
    }).await?;
    let mut message = handle.message().await?;
    let mut players: Vec<Member> = vec![ctx.author_member().await.unwrap().into_owned()];

    while let Some(mci) = message
        .await_component_interactions(ctx.serenity_context())
        .timeout(Duration::from_secs(2 * 60))
        .build()
        .next()
        .await
    {
        let mut db_player = DbUser::fetch_or_insert(&data.db, mci.user.id.0 as i64).await?;
        if db_player.boonbucks < 50 {
            mci.create_interaction_response(ctx.serenity_context(), |ir| {
                ir.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.embed(|e| {
                            Strife::prepare_embed(e).description("You don't have enough Boondollars to join this strife! (50 Boondollars required)")
                        })
                        .ephemeral(true)
                    })
            })
                .await?;
            continue;
        }
        if players.iter().any(|p| p.user.id == mci.user.id) {
            mci.create_interaction_response(ctx.serenity_context(), |ir| {
                ir.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.embed(|e| {
                            Strife::prepare_embed(e).description("You're already in this strife!")
                        })
                        .ephemeral(true)
                    })
            })
            .await?;
            continue;
        }

        if let Some(guild_id) = ctx.guild_id() {
            update_activity(data, mci.user.id, ctx.channel_id(), guild_id).await?;
        }

        db_player.boonbucks -= 50;
        db_player.update(&data.db).await?;

        players.push(mci.member.as_ref().unwrap().clone());
        mci.create_interaction_response(ctx.serenity_context(), |ir| {
            ir.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.embed(|e| {
                        Strife::prepare_embed(e)
                            .description(format!("{} gathers up a crew to **STRIFE**!\n\nPress the button below to join the crew! (50 Boondollars to enter) There are currently {} waiting to fight!\n\nThe battle should start {}", ctx.author(), players.len(), now_in_2_minutes.relative_time()))
                    })
                })
        }).await?;
    }

    let Some(game) = Strife::new(players) else {
        message
            .to_mut()
            .edit(ctx.serenity_context(), |e| {
                e.components(|c| c)
            })
            .await?;
        ctx.send(|m| {
            m.embed(|e| {
                Strife::prepare_embed(e)
                    .description("There weren't enough players to start a strife!")
            })
        }).await?;
        user.boonbucks += 50;
        user.update(&data.db).await?;
        return Ok(());
    };
    let result = game.play().await?;
    match result.result {
        StrifeResult::Wipeout => {
            handle
                .edit(Context::Application(ctx), |e| e.components(|c| c))
                .await?;
            ctx.send(|m| {
                m.embed(|e| {
                    Strife::prepare_embed(e)
                        .description(format!("The battle against the **{}** resulted in a total wipeout! No one survived!", game.enemy_name()))
                })
            })
            .await?;
        }
        StrifeResult::WinSingle => {
            let winner = result.winners.first().unwrap();
            payout_multiple(&data.db, &result.winners, &result).await?;

            handle
                .edit(Context::Application(ctx), |e| e.components(|c| c))
                .await?;
            ctx.send(|m| {
                m.embed(|e| {
                    Strife::prepare_embed(e).description(format!(
                        "In the battle against the **{}**, {} survived and beat the enemy! They won {} Boondollars and {} {} grist!",
                        game.enemy_name(),
                        winner.user,
                        result.boonbucks_per_player.unwrap(),
                        result.grist_per_player.unwrap(),
                        result.grist_type.unwrap()
                    ))
                })
            }).await?;
        }
        StrifeResult::WinHalf => {
            payout_multiple(&data.db, &result.winners, &result).await?;

            let winners = result
                .winners
                .iter()
                .map(|m| m.user.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            handle
                .edit(Context::Application(ctx), |e| e.components(|c| c))
                .await?;
            ctx.send(|m| {
                m.embed(|e| {
                    Strife::prepare_embed(e).description(format!(
                        "The fierce battle against **{}** is over and the following people survived and beat the enemy:\n\n{}\n\nThey won {} Boondollars and {} {} grist each!",
                        game.enemy_name(),
                        winners,
                        result.boonbucks_per_player.unwrap(),
                        result.grist_per_player.unwrap(),
                        result.grist_type.unwrap()
                    ))
                })
            }).await?;
        }
        StrifeResult::WinFull => {
            payout_multiple(&data.db, &result.winners, &result).await?;

            handle
                .edit(Context::Application(ctx), |e| e.components(|c| c))
                .await?;
            ctx.send(|m| {
                m.embed(|e| {
                    Strife::prepare_embed(e).description(format!(
                        "The combatants wiped the floor with the **{}**! They won {} Boondollars and {} {} grist each!",
                        game.enemy_name(),
                        result.boonbucks_per_player.unwrap(),
                        result.grist_per_player.unwrap(),
                        result.grist_type.unwrap()
                    ))
                })
            }).await?;
        }
    }
    set_cooldown(&data.redis_pool, cooldown, 5 * 60).await?;

    Ok(())
}
