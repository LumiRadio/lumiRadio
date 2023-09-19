use poise::serenity_prelude::User;
use sqlx::types::BigDecimal;
use crate::db::DbUser;
use crate::prelude::{ApplicationContext, Error};

#[derive(Debug, Clone, poise::ChoiceParameter)]
pub enum UserParameter {
    #[name = "Watched time"]
    WatchedTime,
    #[name = "Boonbucks"]
    Boonbucks,
    #[name = "Migrated"]
    Migrated,
    #[name = "Amber Grist"]
    AmberGrist,
    #[name = "Amethyst Grist"]
    AmethystGrist,
    #[name = "Artifact Grist"]
    ArtifactGrist,
    #[name = "Caulk Grist"]
    CaulkGrist,
    #[name = "Chalk Grist"]
    ChalkGrist,
    #[name = "Cobalt Grist"]
    CobaltGrist,
    #[name = "Diamond Grist"]
    DiamondGrist,
    #[name = "Garnet Grist"]
    GarnetGrist,
    #[name = "Gold Grist"]
    GoldGrist,
    #[name = "Iodine Grist"]
    IodineGrist,
    #[name = "Marble Grist"]
    MarbleGrist,
    #[name = "Mercury Grist"]
    MercuryGrist,
    #[name = "Quartz Grist"]
    QuartzGrist,
    #[name = "Ruby Grist"]
    RubyGrist,
    #[name = "Rust Grist"]
    RustGrist,
    #[name = "Shale Grist"]
    ShaleGrist,
    #[name = "Sulfur Grist"]
    SulfurGrist,
    #[name = "Tar Grist"]
    TarGrist,
    #[name = "Uranium Grist"]
    UraniumGrist,
    #[name = "Zillium Grist"]
    ZilliumGrist,
}

/// Gets a user's property
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn get(ctx: ApplicationContext<'_>,
                 #[description = "The user to inspect"] user: User,
                    #[description = "The property to inspect"] property: UserParameter
) -> Result<(), Error> {
    let data = ctx.data();

    let db_user = DbUser::fetch_or_insert(&data.db, user.id.0 as i64).await?;
    let value = match property {
        UserParameter::WatchedTime => db_user.watched_time.to_string(),
        UserParameter::Boonbucks => db_user.boonbucks.to_string(),
        UserParameter::Migrated => db_user.migrated.to_string(),
        UserParameter::AmberGrist => db_user.amber.to_string(),
        UserParameter::AmethystGrist => db_user.amethyst.to_string(),
        UserParameter::ArtifactGrist => db_user.artifact.to_string(),
        UserParameter::CaulkGrist => db_user.caulk.to_string(),
        UserParameter::ChalkGrist => db_user.chalk.to_string(),
        UserParameter::CobaltGrist => db_user.cobalt.to_string(),
        UserParameter::DiamondGrist => db_user.diamond.to_string(),
        UserParameter::GarnetGrist => db_user.garnet.to_string(),
        UserParameter::GoldGrist => db_user.gold.to_string(),
        UserParameter::IodineGrist => db_user.iodine.to_string(),
        UserParameter::MarbleGrist => db_user.marble.to_string(),
        UserParameter::MercuryGrist => db_user.mercury.to_string(),
        UserParameter::QuartzGrist => db_user.quartz.to_string(),
        UserParameter::RubyGrist => db_user.ruby.to_string(),
        UserParameter::RustGrist => db_user.rust.to_string(),
        UserParameter::ShaleGrist => db_user.shale.to_string(),
        UserParameter::SulfurGrist => db_user.sulfur.to_string(),
        UserParameter::TarGrist => db_user.tar.to_string(),
        UserParameter::UraniumGrist => db_user.uranium.to_string(),
        UserParameter::ZilliumGrist => db_user.zillium.to_string()
    };

    ctx.send(|m| {
        m.embed(|e| {
            e.title(format!("User {}", user.name))
                .field("Property", property.to_string(), true)
                .field("Value", value, true)
        })
    }).await?;

    Ok(())
}

/// Sets a user's property
#[poise::command(slash_command, ephemeral, owners_only)]
pub async fn set(ctx: ApplicationContext<'_>,
                 #[description = "The user to edit"] user: User,
                 #[description = "The property to set"] property: UserParameter,
                 #[description = "The value to set the property to"] value: String
) -> Result<(), Error> {
    let data = ctx.data();

    let mut db_user = DbUser::fetch_or_insert(&data.db, user.id.0 as i64).await?;
    match property {
        UserParameter::WatchedTime => {
            db_user.watched_time = value.parse::<BigDecimal>()?;
        },
        UserParameter::Boonbucks => {
            db_user.boonbucks = value.parse::<i32>()?;
        },
        UserParameter::Migrated => {
            db_user.migrated = value.parse::<bool>()?;
        },
        UserParameter::AmberGrist => {
            db_user.amber = value.parse::<i32>()?;
        },
        UserParameter::AmethystGrist => {
            db_user.amethyst = value.parse::<i32>()?;
        },
        UserParameter::ArtifactGrist => {
            db_user.artifact = value.parse::<i32>()?;
        },
        UserParameter::CaulkGrist => {
            db_user.caulk = value.parse::<i32>()?;
        },
        UserParameter::ChalkGrist => {
            db_user.chalk = value.parse::<i32>()?;
        },
        UserParameter::CobaltGrist => {
            db_user.cobalt = value.parse::<i32>()?;
        },
        UserParameter::DiamondGrist => {
            db_user.diamond = value.parse::<i32>()?;
        },
        UserParameter::GarnetGrist => {
            db_user.garnet = value.parse::<i32>()?;
        },
        UserParameter::GoldGrist => {
            db_user.gold = value.parse::<i32>()?;
        },
        UserParameter::IodineGrist => {
            db_user.iodine = value.parse::<i32>()?;
        },
        UserParameter::MarbleGrist => {
            db_user.marble = value.parse::<i32>()?;
        },
        UserParameter::MercuryGrist => {
            db_user.mercury = value.parse::<i32>()?;
        },
        UserParameter::QuartzGrist => {
            db_user.quartz = value.parse::<i32>()?;
        },
        UserParameter::RubyGrist => {
            db_user.ruby = value.parse::<i32>()?;
        },
        UserParameter::RustGrist => {
            db_user.rust = value.parse::<i32>()?;
        },
        UserParameter::ShaleGrist => {
            db_user.shale = value.parse::<i32>()?;
        },
        UserParameter::SulfurGrist => {
            db_user.sulfur = value.parse::<i32>()?;
        },
        UserParameter::TarGrist => {
            db_user.tar = value.parse::<i32>()?;
        },
        UserParameter::UraniumGrist => {
            db_user.uranium = value.parse::<i32>()?;
        },
        UserParameter::ZilliumGrist => {
            db_user.zillium = value.parse::<i32>()?;
        }
    }
    db_user.update(&data.db).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Successfully set user property")
                .description(format!("Successfully set {} to {}", property, value))
        })
    }).await?;

    Ok(())
}