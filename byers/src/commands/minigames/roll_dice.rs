use rand::Rng;

use crate::{prelude::{ApplicationContext, Error}, db::{DbServerConfig, DbUser}};

#[poise::command(slash_command, user_cooldown = 300)]
pub async fn roll_dice(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let Some(guild_id) = ctx.guild_id() else {
        return Err(anyhow::anyhow!("This command can only be used in a server"));
    };
    let mut guild_config = DbServerConfig::fetch_or_insert(&data.db, guild_id.0 as i64).await?;
    
    if guild_config.dice_roll == 0 {
        guild_config.dice_roll = 100;
        guild_config.update(&data.db).await?;
    }

    let mut user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;
    if user.boonbucks < 5 {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Insufficient funds")
                    .description("You need at least 5 boonbucks to play")
            })
        }).await?;
        return Ok(());
    }

    // roll 3 6-sided
    let rolls: [u8; 3] = [
        rand::thread_rng().gen_range(1..=6),
        rand::thread_rng().gen_range(1..=6),
        rand::thread_rng().gen_range(1..=6),
    ];
    // stitch them together as one i32
    let roll = rolls[0] as i32 * 100 + rolls[1] as i32 * 10 + rolls[2] as i32;

    // first part of the minigame:
    // if the sum of the rolls is smaller than 12, you get nothing
    // if the sum is between 12 and 15, you get your bet back
    // if the sum is between 16 and 17, you get 2x your bet
    // if the sum is 18, you get 5x your bet
    let sum = rolls.iter().sum::<u8>();
    let winnings = match sum {
        0..=11 => 0,
        12..=15 => 1,
        16..=17 => 2,
        18 => 5,
        _ => unreachable!(),
    } * 5;
    let mut total_winnings = winnings;

    user.boonbucks += winnings - 5;
    user.update(&data.db).await?;

    // second part of the minigame:
    // if the roll is equals to the server's dice roll, you get additional 5x your bet
    // additionally, the server's dice roll is increased by 1
    if roll == guild_config.dice_roll {
        total_winnings += 5 * 5;
        user.boonbucks += 5 * 5;
        guild_config.dice_roll += 1;
        guild_config.update(&data.db).await?;
        user.update(&data.db).await?;

        ctx.send(|m| {
            m.embed(|x| {
                x.title("You won!")
                    .description(format!(r#"You rolled {roll} and won {total_winnings} boonbucks!

                    Additionally, you rolled the server's roll of {}! The next number is {}"#, guild_config.dice_roll - 1, guild_config.dice_roll))
            })
        }).await?;
    } else {
        match total_winnings == 0 {
            true => {
                ctx.send(|m| {
                    m.embed(|x| {
                        x.title("You lost!")
                            .description(format!(r#"You rolled {roll} and lost 5 boonbucks!

                        The server's roll is {}"#, guild_config.dice_roll))
                    })
                }).await?;
            }
            false => {
                    ctx.send(|m| {
                        m.embed(|x| {
                            x.title("You won!")
                                .description(format!(r#"You rolled {roll} and won {total_winnings} boonbucks!

                        The server's roll is {}"#, guild_config.dice_roll))
                        })
                    }).await?;
                }
        }
    }

    Ok(())
}
