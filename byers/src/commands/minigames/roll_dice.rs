use async_trait::async_trait;
use rand::Rng;

use crate::prelude::*;
use crate::{commands::minigames::Minigame, event_handlers::message::update_activity};
use judeharley::{
    communication::ByersUnixStream,
    cooldowns::{is_on_cooldown, set_cooldown, UserCooldownKey},
    db::{DbServerConfig, DbUser},
    prelude::DiscordTimestamp,
};

pub struct DiceRoll {
    server_roll: i32,
    player_roll: [u8; 3],
}

impl DiceRoll {
    pub fn new(server_roll: i32) -> Self {
        Self {
            server_roll,
            player_roll: [
                rand::thread_rng().gen_range(1..=6),
                rand::thread_rng().gen_range(1..=6),
                rand::thread_rng().gen_range(1..=6),
            ],
        }
    }

    pub fn player_roll(&self) -> i32 {
        self.player_roll[0] as i32 * 100
            + self.player_roll[1] as i32 * 10
            + self.player_roll[2] as i32
    }
}

pub enum DiceRollResult {
    Win(i32),
    WinSecret(i32),
    Lose,
}

#[async_trait]
impl Minigame for DiceRoll {
    const NAME: &'static str = "Dice Roll";
    type MinigameResult = DiceRollResult;

    async fn play(&self) -> Result<DiceRollResult, Error> {
        // stitch them together as one i32
        let roll = self.player_roll[0] as i32 * 100
            + self.player_roll[1] as i32 * 10
            + self.player_roll[2] as i32;
        let sum = self.player_roll.iter().sum::<u8>();
        let winnings = match sum {
            0..=10 => 0,
            11..=14 => 2,
            15 => 3,
            16 => 4,
            17 => 5,
            18 => 10,
            _ => unreachable!(),
        } * 5;
        let mut total_winnings = winnings;

        if roll == self.server_roll {
            total_winnings += 75;
            return Ok(DiceRollResult::WinSecret(total_winnings));
        } else if total_winnings > 0 {
            return Ok(DiceRollResult::Win(total_winnings));
        } else {
            return Ok(DiceRollResult::Lose);
        }
    }

    fn command() -> Vec<poise::Command<Data<ByersUnixStream>, anyhow::Error>> {
        vec![roll_dice()]
    }
}

fn roll_over(mut roll: i32) -> i32 {
    if roll == 666 {
        return 111;
    }

    let hundreds = roll / 100;
    let tens = (roll % 100) / 10;
    let ones = roll % 10;

    if ones == 6 {
        if tens == 6 {
            roll = (hundreds + 1) * 100 + 11;
        } else {
            roll = hundreds * 100 + (tens + 1) * 10 + 1;
        }
    } else {
        roll += 1;
    }

    roll
}

fn roll_to_emoji(roll: i32) -> String {
    // transform each digit into the dice emoji
    let hundreds = match roll / 100 {
        1 => "1️⃣",
        2 => "2️⃣",
        3 => "3️⃣",
        4 => "4️⃣",
        5 => "5️⃣",
        6 => "6️⃣",
        _ => unreachable!(),
    };
    let tens = match (roll % 100) / 10 {
        1 => "1️⃣",
        2 => "2️⃣",
        3 => "3️⃣",
        4 => "4️⃣",
        5 => "5️⃣",
        6 => "6️⃣",
        _ => unreachable!(),
    };
    let ones = match roll % 10 {
        1 => "1️⃣",
        2 => "2️⃣",
        3 => "3️⃣",
        4 => "4️⃣",
        5 => "5️⃣",
        6 => "6️⃣",
        _ => unreachable!(),
    };

    format!("{}{}{}", hundreds, tens, ones)
}

/// Roll a dice and win boonbucks
#[poise::command(slash_command, guild_only)]
pub async fn roll_dice(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data();

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err(anyhow::anyhow!("This command can only be used in a server"));
    };
    let mut guild_config = DbServerConfig::fetch_or_insert(&data.db, guild_id.0 as i64).await?;

    if guild_config.dice_roll == 0 {
        guild_config.dice_roll = 111;
        guild_config.update(&data.db).await?;
    }

    let user_cooldown = UserCooldownKey::new(ctx.author().id.0 as i64, "roll_dice");
    if let Some(over) = is_on_cooldown(&data.redis_pool, user_cooldown).await? {
        ctx.send(|m| {
            m.embed(|e| {
                DiceRoll::prepare_embed(e).description(format!(
                    "The dice are being polished for you. You can roll the dice again {}.",
                    over.relative_time()
                ))
            })
        })
        .await?;
        return Ok(());
    }

    let mut user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;
    if user.boonbucks < 5 {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Insufficient funds")
                    .description("You need at least 5 Boondollars to play")
            })
        })
        .await?;
        return Ok(());
    }

    user.boonbucks -= 5;
    user.update(&data.db).await?;

    let game = DiceRoll::new(guild_config.dice_roll);
    let result = game.play().await?;

    match result {
        DiceRollResult::WinSecret(total_winnings) => {
            let old_roll = guild_config.dice_roll;
            guild_config.dice_roll = roll_over(guild_config.dice_roll);
            guild_config.update(&data.db).await?;
            user.boonbucks += total_winnings;
            user.update(&data.db).await?;

            ctx.send(|m| {
                m.embed(|x| {
                    x.title("You won!").description(format!(
                        r#"You rolled {} and won {total_winnings} Boondollars!
    
                        Additionally, you rolled the quest roll of {}! The next number is {}"#,
                        roll_to_emoji(game.player_roll()),
                        roll_to_emoji(old_roll),
                        roll_to_emoji(guild_config.dice_roll)
                    ))
                })
            })
            .await?;
        }
        DiceRollResult::Win(total_winnings) => {
            user.boonbucks += total_winnings;
            user.update(&data.db).await?;

            ctx.send(|m| {
                m.embed(|x| {
                    x.title("You won!").description(format!(
                        r#"You rolled {} and won {total_winnings} Boondollars!

                    The quest roll is {}"#,
                        roll_to_emoji(game.player_roll()),
                        roll_to_emoji(guild_config.dice_roll)
                    ))
                })
            })
            .await?;
        }
        DiceRollResult::Lose => {
            ctx.send(|m| {
                m.embed(|x| {
                    x.title("You lost!").description(format!(
                        r#"You rolled {} and lost 5 Boondollars!

                    The quest roll is {}"#,
                        roll_to_emoji(game.player_roll()),
                        roll_to_emoji(guild_config.dice_roll)
                    ))
                })
            })
            .await?;
        }
    }

    set_cooldown(&data.redis_pool, user_cooldown, 5 * 60).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dice_rollover() {
        assert_eq!(super::roll_over(111), 112);
        assert_eq!(super::roll_over(666), 111);
        assert_eq!(super::roll_over(116), 121);
        assert_eq!(super::roll_over(126), 131);
        assert_eq!(super::roll_over(136), 141);
        assert_eq!(super::roll_over(146), 151);
        assert_eq!(super::roll_over(156), 161);
        assert_eq!(super::roll_over(166), 211);
    }
}
