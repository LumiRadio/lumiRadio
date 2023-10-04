use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use poise::{
    serenity_prelude::{CreateEmbed, Permissions},
    Command,
};

use byers::{
    communication::ByersUnixStream,
    prelude::{Data, Error},
};

pub mod new_slots;
pub mod pvp;
pub mod roll_dice;
pub mod slots;
pub mod strife;

#[async_trait]
pub trait Minigame {
    type MinigameResult;

    const NAME: &'static str;

    async fn play(&self) -> Result<Self::MinigameResult, Error>;

    fn prepare_embed(e: &mut CreateEmbed) -> &mut CreateEmbed {
        e.title(Self::NAME).timestamp(Utc::now())
    }

    fn command() -> Vec<poise::Command<Data<ByersUnixStream>, anyhow::Error>>;
}

pub fn commands() -> Vec<poise::Command<Data<ByersUnixStream>, anyhow::Error>> {
    let mut commands = Vec::new();
    commands.extend(pvp::PvP::command());
    commands.extend(roll_dice::DiceRoll::command());
    commands.extend(new_slots::NewSlots::command());
    commands.extend(strife::Strife::command());
    commands
}

pub fn command() -> Command<Data<ByersUnixStream>, Error> {
    Command {
        name: "minigames".to_string(),
        description: Some("Play a minigame".to_string()),
        slash_action: Some(|_| Box::pin(async move { Ok(()) })),
        prefix_action: None,
        context_menu_action: None,
        subcommands: commands(),
        subcommand_required: true,
        name_localizations: HashMap::new(),
        qualified_name: "minigames".to_string(),
        source_code_name: "minigames".to_string(),
        identifying_name: "minigames".to_string(),
        category: None,
        description_localizations: HashMap::new(),
        help_text: None,
        hide_in_help: false,
        cooldowns: std::sync::Mutex::new(poise::Cooldowns::new(poise::CooldownConfig {
            global: None.map(std::time::Duration::from_secs),
            user: None.map(std::time::Duration::from_secs),
            guild: None.map(std::time::Duration::from_secs),
            channel: None.map(std::time::Duration::from_secs),
            member: None.map(std::time::Duration::from_secs),
        })),
        reuse_response: false,
        default_member_permissions: Permissions::empty(),
        required_permissions: Permissions::empty(),
        required_bot_permissions: Permissions::empty(),
        owners_only: false,
        guild_only: false,
        dm_only: false,
        nsfw_only: false,
        checks: vec![],
        on_error: None,
        parameters: vec![],
        custom_data: Box::new(()),
        aliases: &[],
        invoke_on_edit: false,
        track_deletion: false,
        broadcast_typing: false,
        context_menu_name: None,
        ephemeral: false,
        __non_exhaustive: (),
    }
}
