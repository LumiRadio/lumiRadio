use crate::{
    commands::admin::{control::{control_cmd, queue, skip, volume}, user::{set, get}},
    prelude::{ApplicationContext, Error},
};

pub mod config;
pub mod control;
pub mod import;
pub mod user;

/// Admin commands
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    subcommands("volume", "control_cmd", "skip", "queue"),
    subcommand_required
)]
pub async fn admin(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// User commands
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    subcommands("set", "get"),
    subcommand_required
)]
pub async fn user(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}
