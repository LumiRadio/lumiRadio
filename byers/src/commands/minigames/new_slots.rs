use std::collections::HashMap;
use std::fmt::Display;

use async_trait::async_trait;
use judeharley::sea_orm::Set;
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;
use rand::seq::SliceRandom;
use strum::{EnumIter, IntoEnumIterator};

use crate::prelude::*;
use crate::{commands::minigames::Minigame, event_handlers::message::update_activity};
use judeharley::{
    communication::ByersUnixStream,
    cooldowns::{is_on_cooldown, set_cooldown, UserCooldownKey},
    prelude::DiscordTimestamp,
    Users,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Hash)]
pub enum SlotSymbols {
    Jackpot,
    RedSeven,
    TripleBar,
    DoubleBar,
    Bar,
    Cherry,
}

impl Display for SlotSymbols {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl SlotSymbols {
    pub fn weight(&self) -> u32 {
        match self {
            SlotSymbols::Jackpot => 6,
            SlotSymbols::RedSeven => 8,
            SlotSymbols::TripleBar => 9,
            SlotSymbols::DoubleBar => 11,
            SlotSymbols::Bar => 22,
            SlotSymbols::Cherry => 8,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            SlotSymbols::Jackpot => "☀️",
            SlotSymbols::RedSeven => "🅱️",
            SlotSymbols::TripleBar => "🍋",
            SlotSymbols::DoubleBar => "🍊",
            SlotSymbols::Bar => "🔔",
            SlotSymbols::Cherry => "🍒",
        }
    }

    pub fn is_bar(&self) -> bool {
        matches!(
            self,
            SlotSymbols::Bar | SlotSymbols::DoubleBar | SlotSymbols::TripleBar
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReelSymbol {
    Symbol(SlotSymbols),
    Blank,
}

impl ReelSymbol {
    pub fn is_bar(&self) -> bool {
        match self {
            ReelSymbol::Symbol(s) => s.is_bar(),
            ReelSymbol::Blank => false,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ReelSymbol::Symbol(s) => s.symbol(),
            ReelSymbol::Blank => "⬛",
        }
    }
}

impl PartialEq<SlotSymbols> for ReelSymbol {
    fn eq(&self, other: &SlotSymbols) -> bool {
        match self {
            ReelSymbol::Symbol(s) => s == other,
            ReelSymbol::Blank => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reel {
    pub symbols: Vec<ReelSymbol>,
}

impl Reel {
    pub fn new() -> Self {
        let mut symbol_reel = Vec::with_capacity(64);
        let total_symbol_count = SlotSymbols::iter().map(|s| s.weight()).sum::<u32>();
        for symbol in SlotSymbols::iter() {
            let symbol_count = symbol_reel.iter().filter(|s| **s == symbol).count();
            let symbol_weight = symbol.weight();
            if (symbol_count as u32) < symbol_weight
                && (symbol_count as f32)
                    <= (symbol_weight as f32) / (total_symbol_count as f32)
                        * (symbol_reel.len() as f32)
            {
                symbol_reel.push(ReelSymbol::Symbol(symbol));
            }
        }

        let mut reel = Vec::with_capacity(128);
        for symbol in symbol_reel {
            reel.push(symbol);
            reel.push(ReelSymbol::Blank)
        }

        Self { symbols: reel }
    }

    pub fn spin(&self) -> ReelSymbol {
        let mut rng = rand::thread_rng();
        *self.symbols.choose(&mut rng).unwrap()
    }
}

pub struct SlotMachine {
    pub reels: [Reel; 3],
}

impl SlotMachine {
    pub fn new() -> Self {
        let reel = Reel::new();
        Self {
            reels: [reel.clone(), reel.clone(), reel],
        }
    }

    pub fn spin(&self) -> (Option<u32>, [ReelSymbol; 3]) {
        let reel_1 = self.reels[0].spin();
        let reel_2 = self.reels[1].spin();
        let reel_3 = self.reels[2].spin();
        let reel = [reel_1, reel_2, reel_3];

        let symbol_counts = reel.iter().fold(HashMap::new(), |mut acc, s| {
            *acc.entry(s).or_insert(0) += 1;
            acc
        });

        if *symbol_counts
            .get(&ReelSymbol::Symbol(SlotSymbols::Jackpot))
            .unwrap_or(&0)
            == 3
        {
            return (Some(1200), reel);
        } else if *symbol_counts
            .get(&ReelSymbol::Symbol(SlotSymbols::RedSeven))
            .unwrap_or(&0)
            == 3
        {
            return (Some(200), reel);
        } else if *symbol_counts
            .get(&ReelSymbol::Symbol(SlotSymbols::TripleBar))
            .unwrap_or(&0)
            == 3
        {
            return (Some(100), reel);
        } else if *symbol_counts
            .get(&ReelSymbol::Symbol(SlotSymbols::DoubleBar))
            .unwrap_or(&0)
            == 3
        {
            return (Some(90), reel);
        } else if *symbol_counts
            .get(&ReelSymbol::Symbol(SlotSymbols::Bar))
            .unwrap_or(&0)
            == 3
            || *symbol_counts
                .get(&ReelSymbol::Symbol(SlotSymbols::Cherry))
                .unwrap_or(&0)
                == 3
        {
            return (Some(40), reel);
        } else if reel.iter().all(|s| s.is_bar()) {
            return (Some(10), reel);
        } else if *symbol_counts
            .get(&ReelSymbol::Symbol(SlotSymbols::Cherry))
            .unwrap_or(&0)
            == 2
        {
            return (Some(5), reel);
        } else if *symbol_counts
            .get(&ReelSymbol::Symbol(SlotSymbols::Cherry))
            .unwrap_or(&0)
            == 1
        {
            return (Some(1), reel);
        }

        (None, reel)
    }
}

pub static SLOT_MACHINE: once_cell::sync::Lazy<SlotMachine> =
    once_cell::sync::Lazy::new(SlotMachine::new);

pub struct NewSlots;

#[async_trait]
impl Minigame for NewSlots {
    type MinigameResult = (Option<u32>, [ReelSymbol; 3]);

    const NAME: &'static str = "Slot Machine";

    async fn play(&self) -> Result<Self::MinigameResult, crate::Error> {
        Ok(SLOT_MACHINE.spin())
    }

    fn command() -> Vec<poise::Command<Data<ByersUnixStream>, anyhow::Error>> {
        vec![slots(), slots_info()]
    }
}

/// Shows information about the slot machine
#[poise::command(slash_command)]
pub async fn slots_info(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let jackpot = SlotSymbols::Jackpot.symbol();
    let red_seven = SlotSymbols::RedSeven.symbol();
    let triple_bar = SlotSymbols::TripleBar.symbol();
    let double_bar = SlotSymbols::DoubleBar.symbol();
    let bar = SlotSymbols::Bar.symbol();
    let cherry = SlotSymbols::Cherry.symbol();

    let description = format!(
        r#"# How the Slot Machine works
    
The slot machine has 3 reels, each with 64 symbols. The symbols are weighted, so some symbols are more likely to appear than others. The weights are as follows:

- Jackpot ({jackpot}): 6
- Red Seven ({red_seven}): 8
- Triple Bar ({triple_bar}): 9
- Double Bar ({double_bar}): 11
- Bar ({bar}): 22
- Cherry ({cherry}): 8

Additionally, there are 64 blanks on each reel. The reels are spun, and the symbols that appear on the middle row are used to determine the payout. The payout is determined as follows:

{jackpot} {jackpot} {jackpot}: 1200x
{red_seven} {red_seven} {red_seven}: 200x
{triple_bar} {triple_bar} {triple_bar}: 100x
{double_bar} {double_bar} {double_bar}: 90x
{bar} {bar} {bar}: 40x
{cherry} {cherry} {cherry}: 40x
any 3 of {bar}, {double_bar}, {triple_bar}: 10x
any 2 {cherry}: 5x
any 1 {cherry}: 1x

If none of these combinations are spun, you lose your bet. If you win, you get your bet back multiplied by the payout. For example, if you bet 5 Boondollars and win 40x, you get 200 Boondollars back."#
    );

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Slot Machine")
                .description(description),
        ),
    )
    .await?;

    Ok(())
}

/// Are you feeling lucky?
#[poise::command(slash_command)]
pub async fn slots(
    ctx: ApplicationContext<'_>,
    #[description = "Your bet"]
    #[min = 1]
    #[max = 10]
    bet: i32,
) -> Result<(), Error> {
    let data = ctx.data();

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let user_cooldown = UserCooldownKey::new(ctx.author().id.get() as i64, "slots");
    if let Some(over) = is_on_cooldown(&data.redis_pool, user_cooldown).await? {
        ctx.send(
            CreateReply::default().embed(NewSlots::prepare_embed().description(format!(
                "The slot machine is broken. Come back {}.",
                over.relative_time()
            ))),
        )
        .await?;
        return Ok(());
    }

    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;
    if user.boonbucks < bet {
        ctx.send(CreateReply::default().embed(
            NewSlots::prepare_embed().description("You need at least 1 Boondollar to play slots"),
        ))
        .await?;
        return Ok(());
    }

    let new_boonbucks = user.boonbucks - bet;
    let user = user
        .update(
            judeharley::entities::users::ActiveModel {
                boonbucks: Set(new_boonbucks),
                ..Default::default()
            },
            &data.db,
        )
        .await?;

    let machine = NewSlots;
    let (payout, reels) = machine.play().await?;
    let Some(payout) = payout else {
        ctx.send(
            CreateReply::default().embed(
                NewSlots::prepare_embed()
                    .description("You spin the slot machine and... **lost**!")
                    .field(
                        "Rolls",
                        format!(
                            "{} {} {}",
                            reels[0].symbol(),
                            reels[1].symbol(),
                            reels[2].symbol()
                        ),
                        false,
                    ),
            ),
        )
        .await?;
        return Ok(());
    };

    if payout == 1200 {
        ctx.send(
            CreateReply::default().embed(
                NewSlots::prepare_embed()
                    .description("You spin the slot machine and... **won the jackpot**!")
                    .field(
                        "Rolls",
                        format!(
                            "{} {} {}",
                            reels[0].symbol(),
                            reels[1].symbol(),
                            reels[2].symbol()
                        ),
                        false,
                    )
                    .field("Payout", (payout * bet as u32).to_string(), false),
            ),
        )
        .await?;
    } else if payout == 1 {
        ctx.send(
            CreateReply::default().embed(
                NewSlots::prepare_embed()
                    .description(
                        "You spin the slot machine and... well, at least you got your money back.",
                    )
                    .field(
                        "Rolls",
                        format!(
                            "{} {} {}",
                            reels[0].symbol(),
                            reels[1].symbol(),
                            reels[2].symbol()
                        ),
                        false,
                    )
                    .field("Payout", (payout * bet as u32).to_string(), false),
            ),
        )
        .await?;
    } else {
        ctx.send(
            CreateReply::default().embed(
                NewSlots::prepare_embed()
                    .description("You spin the slot machine and... **won**!")
                    .field(
                        "Rolls",
                        format!(
                            "{} {} {}",
                            reels[0].symbol(),
                            reels[1].symbol(),
                            reels[2].symbol()
                        ),
                        false,
                    )
                    .field("Payout", (payout * bet as u32).to_string(), false),
            ),
        )
        .await?;
    }

    let new_boonbucks = user.boonbucks + payout as i32 * bet;
    user.update(
        judeharley::entities::users::ActiveModel {
            boonbucks: Set(new_boonbucks),
            ..Default::default()
        },
        &data.db,
    )
    .await?;

    set_cooldown(&data.redis_pool, user_cooldown, 5 * 60).await?;

    Ok(())
}
