use crate::commands::admin::control::{control_cmd, queue, reconnect, skip, song_info, volume};

use crate::prelude::*;

pub mod config;
pub mod control;
pub mod import;
pub mod user;

/// Admin commands
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    subcommands("volume", "control_cmd", "skip", "queue", "reconnect", "song_info"),
    subcommand_required
)]
pub async fn admin(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}
