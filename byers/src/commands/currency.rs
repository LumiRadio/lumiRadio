use poise::{serenity_prelude::User, Modal};

use crate::{
    db::{DbSlcbRank, DbUser},
    prelude::*,
};

#[poise::command(slash_command, user_cooldown = 300, ephemeral)]
pub async fn boondollars(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let user = DbUser::fetch_or_insert(&data.db, ctx.author().id.0 as i64).await?;

    // $username - Hours: $hours (Rank #$hourspos) - $currencyname: $points (Rank #$pointspos) - Echeladder: $rank • Next rung in $nxtrankreq hours. - You can check again in 5 minutes.
    let username = &ctx.author().name;
    let hours = user.watched_time.clone();
    let hours_pos = user.fetch_position_in_hours(&data.db).await?;
    let points = user.boonbucks;
    let points_pos = user.fetch_position_in_boonbucks(&data.db).await?;
    let rank_name = DbSlcbRank::fetch_rank_for_user(&user, &data.db).await?;
    let next_rank = DbSlcbRank::fetch_next_rank_for_user(&user, &data.db)
        .await?
        .map(|r| r.hour_requirement)
        .unwrap_or(0);

    let message = format!("{username} - Hours: {hours} (Rank #{hours_pos}) - Boondollars: {points:.0} (Rank #{points_pos}) - Echeladder: {rank_name} • Next rung in {next_rank} hours. - You can check again in 5 minutes.", username = username, hours = hours, hours_pos = hours_pos, rank_name = rank_name, next_rank = next_rank);
    ctx.say(message).await?;

    Ok(())
}

async fn pay_user(
    ctx: ApplicationContext<'_>,
    target_user: User,
    amount: i32,
) -> Result<(), Error> {
    let data = ctx.data();
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

    let transaction = data.db.begin().await?;

    source_db_user.boonbucks -= amount;
    target_db_user.boonbucks += amount;

    source_db_user.update(&data.db).await?;
    target_db_user.update(&data.db).await?;

    transaction.commit().await?;

    ctx.say(format!(
        "You paid {} {} boondollars.",
        target_user.name, amount
    ))
    .await?;

    Ok(())
}

#[poise::command(slash_command, user_cooldown = 300, ephemeral)]
pub async fn pay(ctx: ApplicationContext<'_>, target_user: User, amount: i32) -> Result<(), Error> {
    pay_user(ctx, target_user, amount).await
}

#[derive(Debug, poise::Modal)]
struct PayModal {
    #[name = "Amount"]
    #[placeholder = "123"]
    amount: String,
}

#[poise::command(
    context_menu_command = "Give this user money",
    user_cooldown = 300,
    ephemeral
)]
pub async fn pay_menu(ctx: ApplicationContext<'_>, target_user: User) -> Result<(), Error> {
    let data = PayModal::execute(ctx).await?;
    if let Some(data) = data {
        return pay_user(ctx, target_user, data.amount.parse().unwrap()).await;
    }

    Ok(())
}
