use poise::{serenity_prelude::User, Modal};
use sqlx::types::BigDecimal;

use crate::{
    db::{DbSlcbRank, DbUser},
    event_handlers::message::update_activity,
    prelude::*,
};

/// Check your boondollars and hours
#[poise::command(slash_command, user_cooldown = 300)]
pub async fn boondollars(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;

    // $username - Hours: $hours (Rank #$hourspos) - $currencyname: $points (Rank #$pointspos) - Echeladder: $rank • Next rung in $nxtrankreq hours. - You can check again in 5 minutes.
    let hours = user.watched_time.clone();
    let hours_pos = user.fetch_position_in_hours(&data.db).await?;
    let points = user.boonbucks;
    let points_pos = user.fetch_position_in_boonbucks(&data.db).await?;
    let rank_name = DbSlcbRank::fetch_rank_for_user(&user, &data.db).await?;
    let next_rank = DbSlcbRank::fetch_next_rank_for_user(&user, &data.db)
        .await?
        .map(|r| BigDecimal::from(r.hour_requirement) - user.watched_time)
        .unwrap_or(BigDecimal::from(0));

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Boondollars")
                .field("User", ctx.author().to_string(), false)
                .field("Hours", format!("{hours:.2}"), true)
                .field("Rank", format!("#{}", hours_pos), true)
                .field("\u{200b}", "\u{200b}", false)
                .field("Boondollars", format!("{points:.0}"), true)
                .field("Rank", format!("#{}", points_pos), true)
                .field("\u{200b}", "\u{200b}", false)
                .field("Echeladder", rank_name, true)
                .field("Next rung in", format!("{next_rank:.0} hours"), true)
        })
    })
    .await?;

    Ok(())
}

async fn pay_user(
    ctx: ApplicationContext<'_>,
    target_user: User,
    amount: i32,
) -> Result<(), Error> {
    let data = ctx.data();

    if let Some(guild_id) = ctx.guild_id() {
        update_activity(data, ctx.author().id, ctx.channel_id(), guild_id).await?;
    }

    let mut source_db_user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;
    let mut target_db_user = DbUser::fetch_or_insert(&data.db, target_user.id.0 as i64).await?;

    if source_db_user.boonbucks < amount {
        ctx.say(format!(
            "You don't have enough boondollars to pay that much! You have {}.",
            source_db_user.boonbucks
        ))
        .await?;
        return Ok(());
    }

    if amount < 0 {
        ctx.say("You can't pay negative boondollars!").await?;
        return Ok(());
    }

    let transaction = data.db.begin().await?;

    source_db_user.boonbucks -= amount;
    target_db_user.boonbucks += amount;

    source_db_user.update(&data.db).await?;
    target_db_user.update(&data.db).await?;

    transaction.commit().await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Payment successful").description(format!(
                "You paid {} {} boonbucks.",
                target_user.name, amount
            ))
        })
    })
    .await?;

    Ok(())
}

/// Pay another user some boondollars
#[poise::command(slash_command, user_cooldown = 300)]
pub async fn pay(ctx: ApplicationContext<'_>, target_user: User, amount: i32) -> Result<(), Error> {
    pay_user(ctx, target_user, amount).await
}

#[derive(Debug, poise::Modal)]
struct PayModal {
    #[name = "Amount"]
    #[placeholder = "123"]
    amount: String,
}

#[poise::command(context_menu_command = "Give this user money", user_cooldown = 300)]
pub async fn pay_menu(ctx: ApplicationContext<'_>, target_user: User) -> Result<(), Error> {
    let data = PayModal::execute(ctx).await?;
    if let Some(data) = data {
        return pay_user(ctx, target_user, data.amount.parse().unwrap()).await;
    }

    Ok(())
}
