use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};

use crate::entities::server_config::*;
use crate::prelude::JudeHarleyError;

#[derive(Default)]
pub struct Params {
    pub slot_jackpot: Option<i32>,
    pub dice_roll: Option<i32>,
}

impl Model {
    pub async fn get(id: u64, db: &DatabaseConnection) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find_by_id(id as i64)
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_or_insert(id: u64, db: &DatabaseConnection) -> Result<Self, JudeHarleyError> {
        if let Some(server_config) = Self::get(id, db).await? {
            return Ok(server_config);
        }

        ActiveModel {
            id: Set(id as i64),
            slot_jackpot: Set(0),
            dice_roll: Set(111),
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

        if let Some(slot_jackpot) = params.slot_jackpot {
            config.slot_jackpot = Set(slot_jackpot);
        }

        if let Some(dice_roll) = params.dice_roll {
            config.dice_roll = Set(dice_roll);
        }

        config.update(db).await.map_err(Into::into)
    }
}
