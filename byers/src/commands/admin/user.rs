use crate::prelude::*;
use judeharley::{sea_orm::Set, Users};
use poise::{
    serenity_prelude::{CreateEmbed, User},
    CreateReply,
};

/// User commands
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    subcommands("set", "get"),
    subcommand_required
)]
pub async fn user(_: ApplicationContext<'_>) -> Result<(), Error> {
    Ok(())
}

#[derive(Debug, Clone, poise::ChoiceParameter, strum::Display)]
pub enum UserParameter {
    #[name = "Watched time"]
    WatchedTime,
    #[name = "Boonbucks"]
    Boonbucks,
    #[name = "Migrated"]
    Migrated,
}

#[derive(Debug, Clone, poise::ChoiceParameter, strum::Display)]
pub enum UserGristParameter {
    #[name = "Amber Grist"]
    Amber,
    #[name = "Amethyst Grist"]
    Amethyst,
    #[name = "Artifact Grist"]
    Artifact,
    #[name = "Caulk Grist"]
    Caulk,
    #[name = "Chalk Grist"]
    Chalk,
    #[name = "Cobalt Grist"]
    Cobalt,
    #[name = "Diamond Grist"]
    Diamond,
    #[name = "Garnet Grist"]
    Garnet,
    #[name = "Gold Grist"]
    Gold,
    #[name = "Iodine Grist"]
    Iodine,
    #[name = "Marble Grist"]
    Marble,
    #[name = "Mercury Grist"]
    Mercury,
    #[name = "Quartz Grist"]
    Quartz,
    #[name = "Ruby Grist"]
    Ruby,
    #[name = "Rust Grist"]
    Rust,
    #[name = "Shale Grist"]
    Shale,
    #[name = "Sulfur Grist"]
    Sulfur,
    #[name = "Tar Grist"]
    Tar,
    #[name = "Uranium Grist"]
    Uranium,
    #[name = "Zillium Grist"]
    Zillium,
}

/// Gets the grist of a user
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn get_grist(
    ctx: ApplicationContext<'_>,
    #[description = "The user to inspect"] user: User,
    #[description = "The grist type to inspect"] grist_type: UserGristParameter,
) -> Result<(), Error> {
    let data = ctx.data();

    let db_user = Users::get_or_insert(user.id.get(), &data.db).await?;
    let value = match grist_type {
        UserGristParameter::Amber => db_user.amber.to_string(),
        UserGristParameter::Amethyst => db_user.amethyst.to_string(),
        UserGristParameter::Artifact => db_user.artifact.to_string(),
        UserGristParameter::Caulk => db_user.caulk.to_string(),
        UserGristParameter::Chalk => db_user.chalk.to_string(),
        UserGristParameter::Cobalt => db_user.cobalt.to_string(),
        UserGristParameter::Diamond => db_user.diamond.to_string(),
        UserGristParameter::Garnet => db_user.garnet.to_string(),
        UserGristParameter::Gold => db_user.gold.to_string(),
        UserGristParameter::Iodine => db_user.iodine.to_string(),
        UserGristParameter::Marble => db_user.marble.to_string(),
        UserGristParameter::Mercury => db_user.mercury.to_string(),
        UserGristParameter::Quartz => db_user.quartz.to_string(),
        UserGristParameter::Ruby => db_user.ruby.to_string(),
        UserGristParameter::Rust => db_user.rust.to_string(),
        UserGristParameter::Shale => db_user.shale.to_string(),
        UserGristParameter::Sulfur => db_user.sulfur.to_string(),
        UserGristParameter::Tar => db_user.tar.to_string(),
        UserGristParameter::Uranium => db_user.uranium.to_string(),
        UserGristParameter::Zillium => db_user.zillium.to_string(),
    };

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title(format!("User {}", user.name))
                .field("Property", grist_type.to_string(), true)
                .field("Value", value, true),
        ),
    )
    .await?;

    Ok(())
}

/// Gets a user's property
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn get(
    ctx: ApplicationContext<'_>,
    #[description = "The user to inspect"] user: User,
    #[description = "The property to inspect"] property: UserParameter,
) -> Result<(), Error> {
    let data = ctx.data();

    let db_user = Users::get_or_insert(user.id.get(), &data.db).await?;
    let value = match property {
        UserParameter::WatchedTime => db_user.watched_time.to_string(),
        UserParameter::Boonbucks => db_user.boonbucks.to_string(),
        UserParameter::Migrated => db_user.migrated.to_string(),
    };

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title(format!("User {}", user.name))
                .field("Property", property.to_string(), true)
                .field("Value", value, true),
        ),
    )
    .await?;

    Ok(())
}

/// Sets a user's grist
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn set_grist(
    ctx: ApplicationContext<'_>,
    #[description = "The user to edit"] user: User,
    #[description = "The grist type to edit"] grist_type: UserGristParameter,
    #[description = "The value to set the grist to"] value: i32,
) -> Result<(), Error> {
    let data = ctx.data();

    let db_user = Users::get_or_insert(user.id.get(), &data.db).await?;
    let mut params = judeharley::entities::users::ActiveModel::default();
    match grist_type {
        UserGristParameter::Amber => {
            params.amber = Set(value);
        }
        UserGristParameter::Amethyst => {
            params.amethyst = Set(value);
        }
        UserGristParameter::Artifact => {
            params.artifact = Set(value);
        }
        UserGristParameter::Caulk => {
            params.caulk = Set(value);
        }
        UserGristParameter::Chalk => {
            params.chalk = Set(value);
        }
        UserGristParameter::Cobalt => {
            params.cobalt = Set(value);
        }
        UserGristParameter::Diamond => {
            params.diamond = Set(value);
        }
        UserGristParameter::Garnet => {
            params.garnet = Set(value);
        }
        UserGristParameter::Gold => {
            params.gold = Set(value);
        }
        UserGristParameter::Iodine => {
            params.iodine = Set(value);
        }
        UserGristParameter::Marble => {
            params.marble = Set(value);
        }
        UserGristParameter::Mercury => {
            params.mercury = Set(value);
        }
        UserGristParameter::Quartz => {
            params.quartz = Set(value);
        }
        UserGristParameter::Ruby => {
            params.ruby = Set(value);
        }
        UserGristParameter::Rust => {
            params.rust = Set(value);
        }
        UserGristParameter::Shale => {
            params.shale = Set(value);
        }
        UserGristParameter::Sulfur => {
            params.sulfur = Set(value);
        }
        UserGristParameter::Tar => {
            params.tar = Set(value);
        }
        UserGristParameter::Uranium => {
            params.uranium = Set(value);
        }
        UserGristParameter::Zillium => {
            params.zillium = Set(value);
        }
    }
    db_user.update(params, &data.db).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Successfully set user grist")
                .description(format!("Successfully set {} to {}", grist_type, value)),
        ),
    )
    .await?;

    Ok(())
}

/// Sets a user's property
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn set(
    ctx: ApplicationContext<'_>,
    #[description = "The user to edit"] user: User,
    #[description = "The property to set"] property: UserParameter,
    #[description = "The value to set the property to"] value: String,
) -> Result<(), Error> {
    let data = ctx.data();

    let db_user = Users::get_or_insert(user.id.get(), &data.db).await?;
    let mut params = judeharley::entities::users::ActiveModel::default();
    match property {
        UserParameter::WatchedTime => {
            params.watched_time = Set(value.parse::<i64>()?);
        }
        UserParameter::Boonbucks => {
            params.boonbucks = Set(value.parse::<i32>()?);
        }
        UserParameter::Migrated => {
            params.migrated = Set(value.parse::<bool>()?);
        }
    }
    db_user.update(params, &data.db).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Successfully set user property")
                .description(format!("Successfully set {} to {}", property, value)),
        ),
    )
    .await?;

    Ok(())
}
