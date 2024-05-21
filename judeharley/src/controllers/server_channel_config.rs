use sea_orm::{prelude::*, IntoActiveModel, Set};

use crate::entities::server_channel_config::*;
use crate::prelude::JudeHarleyError;

#[derive(Default)]
pub struct UpdateParams {
    pub allow_watch_time_accumulation: Option<bool>,
    pub allow_point_accumulation: Option<bool>,
    pub hydration_reminder: Option<bool>,
}

impl Model {
    pub async fn get(id: u64, db: &DatabaseConnection) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find_by_id(id as i64)
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_or_insert(id: u64, db: &DatabaseConnection) -> Result<Self, JudeHarleyError> {
        if let Some(server_channel_config) = Self::get(id, db).await? {
            return Ok(server_channel_config);
        }

        ActiveModel {
            id: Set(id as i64),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(Into::into)
    }

    pub async fn update(
        self,
        params: UpdateParams,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        let mut config = self.into_active_model();

        if let Some(allow_watch_time_accumulation) = params.allow_watch_time_accumulation {
            config.allow_watch_time_accumulation = Set(allow_watch_time_accumulation);
        }

        if let Some(allow_point_accumulation) = params.allow_point_accumulation {
            config.allow_point_accumulation = Set(allow_point_accumulation);
        }

        if let Some(hydration_reminder) = params.hydration_reminder {
            config.hydration_reminder = Set(hydration_reminder);
        }

        config.update(db).await.map_err(Into::into)
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
