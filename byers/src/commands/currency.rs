use poise::{
    serenity_prelude::{CreateEmbed, User},
    CreateReply, Modal,
};

use judeharley::{
    sea_orm::{ActiveModelTrait, DbErr, IntoActiveModel, TransactionTrait},
    Decimal, SlcbRank, Users,
};

use crate::{event_handlers::message::update_activity, prelude::*};

/// Check your Boondollars and hours
#[poise::command(slash_command, user_cooldown = 300)]
pub async fn boondollars(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = ctx.data();

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;

    // $username - Hours: $hours (Rank #$hourspos) - $currencyname: $points (Rank #$pointspos) - Echeladder: $rank • Next rung in $nxtrankreq hours. - You can check again in 5 minutes.
    let hours = user.watched_time;
    let rounded_hours = (hours * Decimal::from(12)).trunc_with_scale(0) / Decimal::from(12);
    let hours_pos = user.hour_position(&data.db).await?;
    let points = user.boonbucks;
    let points_pos = user.boondollar_position(&data.db).await?;
    let rank_name = SlcbRank::get_rank_for_user(&user, &data.db).await?;
    let next_rank = SlcbRank::get_next_rank_for_user(&user, &data.db)
        .await?
        .map(|r| Decimal::from(r.hour_requirement) - user.watched_time)
        .unwrap_or(Decimal::from(0));

    let message = format!("{username} - Hours: {hours:.2} (Rank #{hours_pos}) - Boondollars: {points:.0} (Rank #{points_pos}) - Echeladder: {rank_name} • Next rung in {next_rank:.0} hours. - You can check again in 5 minutes.", username = ctx.author().name, hours = rounded_hours, hours_pos = hours_pos, rank_name = rank_name, next_rank = next_rank);
    ctx.say(message).await?;

    Ok(())
}

async fn pay_user(
    ctx: ApplicationContext<'_>,
    target_user: User,
    amount: i32,
) -> Result<(), Error> {
    let data = ctx.data();

    update_activity(data, ctx.author().id, ctx.channel_id()).await?;

    let source_db_user = Users::get_or_insert(ctx.author().id.get(), &data.db).await?;
    let target_db_user = Users::get_or_insert(target_user.id.get(), &data.db).await?;

    if source_db_user.boonbucks < amount {
        ctx.send(
            CreateReply::default().embed(CreateEmbed::new().title("Payment failed").description(
                format!(
                    "You don't have enough Boondollars to pay that much! You have {}.",
                    source_db_user.boonbucks
                ),
            )),
        )
        .await?;
        return Ok(());
    }

    if amount < 0 {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Payment failed")
                    .description("You can't pay negative boondollars!".to_string()),
            ),
        )
        .await?;
        return Ok(());
    }

    if source_db_user.id == target_db_user.id {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Payment successful")
                    .description(format!(
                        "Congratulations. You just paid yourself {} Boondollars.",
                        amount
                    )),
            ),
        )
        .await?;
        return Ok(());
    }

    let source_boons = source_db_user.boonbucks;
    let target_boons = target_db_user.boonbucks;
    data.db
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                let mut source_active = source_db_user.into_active_model();
                let mut target_active = target_db_user.into_active_model();

                source_active.boonbucks = judeharley::sea_orm::Set(source_boons - amount);
                target_active.boonbucks = judeharley::sea_orm::Set(target_boons + amount);

                source_active.save(txn).await?;
                target_active.save(txn).await?;

                Ok(())
            })
        })
        .await?;

    ctx.send(
        CreateReply::default().embed(CreateEmbed::new().title("Payment successful").description(
            format!("You paid {} {} Boondollars.", target_user.name, amount),
        )),
    )
    .await?;

    Ok(())
}

/// Pay another user some Boondollars
#[poise::command(slash_command, rename = "give", user_cooldown = 300)]
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
