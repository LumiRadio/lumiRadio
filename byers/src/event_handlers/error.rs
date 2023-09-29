use chrono::Utc;
use tracing::error;
use tracing_unwrap::ResultExt;

use crate::{
    communication::ByersUnixStream,
    prelude::{Context, Data, DiscordTimestamp, Error},
};

type FrameworkError<'a> = poise::FrameworkError<'a, Data<ByersUnixStream>, Error>;

async fn send_cooldown_embed(
    ctx: Context<'_>,
    remaining_cooldown: core::time::Duration,
) -> Result<(), Error> {
    ctx.send(|m| {
        m.embed(|e| {
            e.title("You are too fast!").description(format!(
                "You can use that command again {}.",
                (Utc::now().naive_utc() + chrono::Duration::from_std(remaining_cooldown).unwrap())
                    .relative_time()
            ))
        })
        .ephemeral(true)
    })
    .await?;

    Ok(())
}

pub async fn on_error(error: FrameworkError<'_>) -> Result<(), Error> {
    match error {
        FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
        } => {
            send_cooldown_embed(ctx, remaining_cooldown).await?;
        }
        _ => {
            let result = poise::builtins::on_error(error).await;
            if let Err(error) = result {
                error!("Discord error: {}", error);
            }
        }
    }

    Ok(())
}
