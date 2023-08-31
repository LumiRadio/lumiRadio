use anyhow::anyhow;
use rand::seq::IteratorRandom;
use strum::IntoEnumIterator;


use crate::{
    db::{DbServerConfig, DbUser},
    prelude::*,
};

#[derive(strum::EnumIter, PartialEq, Eq, Clone, Copy)]
enum SlotReel {
    Cherry,
    Lemon,
    Orange,
    Plum,
    Bell,
    Bar,
    Seven,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PayoutOptions {
    Money(i32),
    Jackpot,
    Nothing,
}

trait Roll {
    fn determine_payout(&self) -> PayoutOptions;
}

impl Roll for [SlotReel; 3] {
    fn determine_payout(&self) -> PayoutOptions {
        if self[0] == SlotReel::Bar && self[1] == SlotReel::Bar && self[2] == SlotReel::Bar {
            PayoutOptions::Money(250)
        } else if self[0] == SlotReel::Bell
            && self[1] == SlotReel::Bell
            && (self[2] == SlotReel::Bell || self[2] == SlotReel::Bar)
        {
            PayoutOptions::Money(20)
        } else if self[0] == SlotReel::Plum
            && self[1] == SlotReel::Plum
            && (self[2] == SlotReel::Plum || self[2] == SlotReel::Bar)
        {
            PayoutOptions::Money(14)
        } else if self[0] == SlotReel::Orange
            && self[1] == SlotReel::Orange
            && (self[2] == SlotReel::Orange || self[2] == SlotReel::Bar)
        {
            PayoutOptions::Money(10)
        } else if self[0] == SlotReel::Cherry
            && self[1] == SlotReel::Cherry
            && self[2] == SlotReel::Cherry
        {
            PayoutOptions::Money(7)
        } else if self[0] == SlotReel::Cherry && self[1] == SlotReel::Cherry {
            PayoutOptions::Money(5)
        } else if self[0] == SlotReel::Cherry {
            PayoutOptions::Money(2)
        } else if self[0] == SlotReel::Seven
            && self[1] == SlotReel::Seven
            && self[2] == SlotReel::Seven
        {
            PayoutOptions::Jackpot
        } else {
            PayoutOptions::Nothing
        }
    }
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

#[poise::command(slash_command, user_cooldown = 300)]
pub async fn slots(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let Some(guild_id) = ctx.guild_id() else {
        return Err(anyhow!("This command can only be used in a server"));
    };
    let mut server_config = DbServerConfig::fetch_or_insert(&data.db, guild_id.0 as i64).await?;

    let mut user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;
    if user.boonbucks < 5 {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Not enough Boonbucks")
                    .description("You need at least 5 boonbucks to play slots")
                    .color(0xFF0000)
            })
        })
        .await?;
        return Ok(());
    }
    user.boonbucks -= 5;
    user.update(&data.db).await?;
    server_config.slot_jackpot += 5;
    server_config.update(&data.db).await?;

    let rolls = SlotReel::generate_roll();
    let payout = rolls.determine_payout();

    match payout {
        PayoutOptions::Money(amount) => {
            user.boonbucks += amount;
            user.update(&data.db).await?;
        }
        PayoutOptions::Jackpot => {
            user.boonbucks += server_config.slot_jackpot;
            server_config.slot_jackpot = 0;
            user.update(&data.db).await?;
            server_config.update(&data.db).await?;
        }
        PayoutOptions::Nothing => {}
    }

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Slot Machine")
                .description("You paid 5 Boonbucks to play slots and got...")
                .field(
                    "Rolls",
                    format!(
                        "{} {} {}",
                        rolls[0].emoji(),
                        rolls[1].emoji(),
                        rolls[2].emoji()
                    ),
                    false,
                )
                .field(
                    "Payout",
                    match payout {
                        PayoutOptions::Money(amount) => format!("{} Boonbucks", amount),
                        PayoutOptions::Jackpot => {
                            format!("{} Boonbucks", server_config.slot_jackpot)
                        }
                        PayoutOptions::Nothing => "Nothing".to_string(),
                    },
                    false,
                )
                .color(0x00FF00)
        })
    })
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    mod slots {
        use super::super::*;

        #[test]
        fn test_three_bar() {
            let rolls = [SlotReel::Bar, SlotReel::Bar, SlotReel::Bar];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Money(250));
        }

        #[test]
        fn test_three_bell() {
            let rolls = [SlotReel::Bell, SlotReel::Bell, SlotReel::Bell];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Money(20));
        }

        #[test]
        fn test_three_plum() {
            let rolls = [SlotReel::Plum, SlotReel::Plum, SlotReel::Plum];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Money(14));
        }

        #[test]
        fn test_three_orange() {
            let rolls = [SlotReel::Orange, SlotReel::Orange, SlotReel::Orange];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Money(10));
        }

        #[test]
        fn test_three_cherry() {
            let rolls = [SlotReel::Cherry, SlotReel::Cherry, SlotReel::Cherry];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Money(7));
        }

        #[test]
        fn test_two_cherry() {
            let rolls = [SlotReel::Cherry, SlotReel::Cherry, SlotReel::Orange];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Money(5));
        }

        #[test]
        fn test_one_cherry() {
            let rolls = [SlotReel::Cherry, SlotReel::Orange, SlotReel::Orange];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Money(2));
        }

        #[test]
        fn test_three_seven() {
            let rolls = [SlotReel::Seven, SlotReel::Seven, SlotReel::Seven];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Jackpot);
        }

        #[test]
        fn test_two_seven() {
            let rolls = [SlotReel::Seven, SlotReel::Seven, SlotReel::Orange];
            assert_eq!(rolls.determine_payout(), PayoutOptions::Nothing);
        }
    }
}
