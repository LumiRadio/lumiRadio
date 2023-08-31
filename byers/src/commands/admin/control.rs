use crate::{prelude::{ApplicationContext, Error}, communication::LiquidsoapCommunication};

#[poise::command(slash_command, ephemeral, owners_only, subcommands("volume", "control_cmd", "skip"))]
pub async fn admin(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn control_cmd(ctx: ApplicationContext<'_>, #[description = "Command to send"] command: String) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    let mut response = comms.send_wait(&command).await?.trim().to_string();
    response.truncate(2000);
    ctx.send(|m| {
        m.embed(|e| {
            e.title("Command Response")
                .description(format!("```\n{}\n```", response))
                .field("Command", command, false)
        })
    }).await?;

    Ok(())
}

#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn volume(ctx: ApplicationContext<'_>, 
    #[description = "Volume to set"]
    #[min = 0]
    #[max = 100]
    volume: Option<i32>,
) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    let Some(volume) = volume else {
        let set_volume = comms.send_wait("var.get volume").await?;
        let set_volume = set_volume.trim().parse::<f32>().unwrap_or(0.0);
        ctx.send(|m| {
            m.embed(|e| {
                e.title("Volume")
                    .description(format!("Volume is set to {}%", (set_volume * 100.0) as i32))
            })
        }).await?;
        return Ok(());
    };

    let _ = comms.send_wait(&format!("var.set volume {}", volume as f32 / 100.0)).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Volume Set")
                .description(format!("Volume set to {}%", volume))
        })
    }).await?;

    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
enum SkipType {
    Radio,
    SongRequest,
    PriorityRequest,
}

#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn skip(ctx: ApplicationContext<'_>, #[description = "What to skip"] skip_type: SkipType) -> Result<(), Error> {
    let mut comms = ctx.data.comms.lock().await;

    let command = match skip_type {
        SkipType::Radio => "lumiradio.skip",
        SkipType::SongRequest => "srq.skip",
        SkipType::PriorityRequest => "prioq.skip",
    };

    let _ = comms.send_wait(command).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Skipped")
                .description(format!("Skipped {}", skip_type))
        })
    }).await?;

    Ok(())
}

