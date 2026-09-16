use crate::commands::admin::control::{
    control_cmd, generate_playlist, queue, reconnect, reindex, skip, song_info, song_tag, volume,
};

use crate::commands::admin::import::import_manually;
use crate::prelude::*;

pub mod config;
pub mod control;
pub mod import;
pub mod ranks;
pub mod user;

/// Admin commands
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    subcommands(
        "volume",
        "control_cmd",
        "skip",
        "queue",
        "reconnect",
        "song_info",
        "import_manually",
        "reindex",
        "generate_playlist",
        "song_tag",
    ),
    subcommand_required
)]
pub async fn admin(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}
