use sea_orm::{ActiveValue, Set, prelude::*};

use crate::entities::server_channel_config::*;
use crate::prelude::JudeHarleyError;

impl Model {
    pub async fn get(id: u64, db: &DatabaseConnection) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find_by_id(id as i64)
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_or_insert(
        id: u64,
        server_id: u64,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        if let Some(server_channel_config) = Self::get(id, db).await? {
            return Ok(server_channel_config);
        }

        ActiveModel {
            id: Set(id as i64),
            server_id: Set(server_id as i64),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(Into::into)
    }

    pub async fn update(
        &self,
        mut params: ActiveModel,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        params.id = ActiveValue::unchanged(self.id);

        Entity::update(params)
            .filter(Column::Id.eq(self.id))
            .exec(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_all_hydration_channels(
        db: &DatabaseConnection,
    ) -> Result<Vec<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::HydrationReminder.eq(true))
            .all(db)
            .await
            .map_err(Into::into)
    }
}
