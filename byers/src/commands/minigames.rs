use rand::seq::IteratorRandom;
use strum::IntoEnumIterator;
use tracing_unwrap::ResultExt;

use crate::{
    db::{DbServerConfig, DbUser},
    prelude::*,
};

#[derive(strum::EnumIter, PartialEq, Eq)]
enum SlotReel {
    Cherry,
    Lemon,
    Orange,
    Plum,
    Bell,
    Bar,
    Seven,
}

enum PayoutOptions {
    Money(i32),
    Jackpot,
    Nothing,
}

impl SlotReel {
    fn emoji(&self) -> &'static str {
        match self {
            SlotReel::Cherry => "🍒",
            SlotReel::Lemon => "🍋",
            SlotReel::Orange => "🍊",
            SlotReel::Plum => "🍇",
            SlotReel::Bell => "🔔",
            SlotReel::Bar => "🅱️",
            SlotReel::Seven => "☀️",
        }
    }

    fn generate_roll() -> [SlotReel; 3] {
        let mut rng = rand::thread_rng();
        [
            SlotReel::iter().choose(&mut rng).unwrap(),
            SlotReel::iter().choose(&mut rng).unwrap(),
            SlotReel::iter().choose(&mut rng).unwrap(),
        ]
    }
}

fn determine_payout(rolls: &[SlotReel; 3]) -> PayoutOptions {
    if rolls[0] == SlotReel::Bar && rolls[1] == SlotReel::Bar && rolls[2] == SlotReel::Bar {
        PayoutOptions::Money(250)
    } else if rolls[0] == SlotReel::Bell
        && rolls[1] == SlotReel::Bell
        && (rolls[2] == SlotReel::Bell || rolls[2] == SlotReel::Bar)
    {
        PayoutOptions::Money(20)
    } else if rolls[0] == SlotReel::Plum
        && rolls[1] == SlotReel::Plum
        && (rolls[2] == SlotReel::Plum || rolls[2] == SlotReel::Bar)
    {
        PayoutOptions::Money(14)
    } else if rolls[0] == SlotReel::Orange
        && rolls[1] == SlotReel::Orange
        && (rolls[2] == SlotReel::Orange || rolls[2] == SlotReel::Bar)
    {
        PayoutOptions::Money(10)
    } else if rolls[0] == SlotReel::Cherry
        && rolls[1] == SlotReel::Cherry
        && rolls[2] == SlotReel::Cherry
    {
        PayoutOptions::Money(7)
    } else if rolls[0] == SlotReel::Cherry && rolls[1] == SlotReel::Cherry {
        PayoutOptions::Money(5)
    } else if rolls[0] == SlotReel::Cherry {
        PayoutOptions::Money(2)
    } else if rolls[0] == SlotReel::Seven
        && rolls[1] == SlotReel::Seven
        && rolls[2] == SlotReel::Seven
    {
        PayoutOptions::Jackpot
    } else {
        PayoutOptions::Nothing
    }
}

#[poise::command(slash_command, user_cooldown = 300, ephemeral)]
pub async fn slots(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let mut user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;

    if user.boonbucks < 5 {
        ctx.say("You don't have enough boonbucks to play slots!")
            .await?;
        return Ok(());
    }
    user.boonbucks -= 5;
    user.update(&data.db).await?;

    let mut server_config =
        DbServerConfig::fetch_or_insert(&data.db, ctx.guild_id().unwrap().0 as i64).await?;

    server_config.slot_jackpot += 5;
    server_config.update(&data.db).await?;
    let slot_jackpot = server_config.slot_jackpot;

    let reel = SlotReel::generate_roll();
    let payout = determine_payout(&reel);

    match payout {
        PayoutOptions::Money(amount) => {
            user.boonbucks += amount;
            user.update(&data.db).await?;
        }
        PayoutOptions::Jackpot => {
            user.boonbucks += server_config.slot_jackpot;
            user.update(&data.db).await?;

            server_config.slot_jackpot = 0;
            server_config.update(&data.db).await?;
        }
        PayoutOptions::Nothing => {}
    }

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Slots");
            e.description(format!(
                "{}{}{}\n\n{}",
                reel[0].emoji(),
                reel[1].emoji(),
                reel[2].emoji(),
                match payout {
                    PayoutOptions::Money(amount) => format!("You won {} boonbucks!", amount),
                    PayoutOptions::Jackpot => {
                        format!("You won the jackpot! You won {} boonbucks!", slot_jackpot)
                    }
                    PayoutOptions::Nothing => "You didn't win anything.".to_string(),
                }
            ));
            e
        })
        .ephemeral(false)
    })
    .await
    .expect_or_log("Failed to send message");

    Ok(())
}
