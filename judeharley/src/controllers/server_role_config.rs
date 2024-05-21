use sea_orm::{prelude::*, IntoActiveModel, Set};

use crate::entities::server_role_config::*;
use crate::prelude::JudeHarleyError;

pub struct Params {
    pub minimum_hours: Option<i32>,
}

impl Model {
    pub async fn get_by_role(
        role_id: u64,
        guild_id: u64,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::GuildId.eq(guild_id as i64))
            .filter(Column::RoleId.eq(role_id as i64))
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_by_role(
        role_id: u64,
        guild_id: u64,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        Entity::delete_many()
            .filter(Column::GuildId.eq(guild_id as i64))
            .filter(Column::RoleId.eq(role_id as i64))
            .exec(db)
            .await
            .map_err(Into::into)
            .map(|_| ())
    }

    pub async fn get_or_insert(
        role_id: u64,
        guild_id: u64,
        minimum_hours: u32,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        if let Some(server_role_config) = Self::get_by_role(role_id, guild_id, db).await? {
            return Ok(server_role_config);
        }

        ActiveModel {
            guild_id: Set(guild_id as i64),
            role_id: Set(role_id as i64),
            minimum_hours: Set(minimum_hours as i32),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(Into::into)
    }

    pub async fn update(
        self,
        params: Params,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        let mut config = self.into_active_model();

        if let Some(minimum_hours) = params.minimum_hours {
            config.minimum_hours = Set(minimum_hours);
        }

        config.update(db).await.map_err(Into::into)
    }
}
