use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entities::server_config::*;
use crate::prelude::JudeHarleyError;

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
        &self,
        params: ActiveModel,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        Entity::update(params)
            .filter(Column::Id.eq(self.id))
            .exec(db)
            .await
            .map_err(Into::into)
    }
}
