use rand::seq::IteratorRandom;
use strum::IntoEnumIterator;

use crate::prelude::*;

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
}

#[poise::command(slash_command, user_cooldown = 300, ephemeral)]
pub async fn slots(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}
