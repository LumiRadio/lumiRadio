use chrono::NaiveDateTime;
use sea_orm::{prelude::*, Iterable, QueryOrder, QuerySelect, Set};

use crate::controllers::CountQuery;
use crate::custom_entities::songs::{
    Column as SongColumn, Entity as SongEntity, Model as SongModel,
};
use crate::entities::played_songs::*;
use crate::prelude::JudeHarleyError;

impl Model {
    pub async fn insert(song: &SongModel, db: &DatabaseConnection) -> Result<(), JudeHarleyError> {
        ActiveModel {
            song_id: Set(song.file_hash.clone()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(())
    }

    pub async fn get_playing_at(
        timestamp: NaiveDateTime,
        db: &DatabaseConnection,
    ) -> Result<Option<SongModel>, JudeHarleyError> {
        Entity::find()
            .select_only()
            .columns(SongColumn::iter())
            .join(
                sea_orm::JoinType::InnerJoin,
                Entity::belongs_to(SongEntity)
                    .from(Column::SongId)
                    .to(SongColumn::FileHash)
                    .into(),
            )
            .filter(Column::PlayedAt.lte(timestamp))
            .order_by_desc(Column::PlayedAt)
            .limit(1)
            .into_model::<SongModel>()
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_last_played(
        db: &DatabaseConnection,
    ) -> Result<Option<SongModel>, JudeHarleyError> {
        Entity::find()
            .select_only()
            .columns(SongColumn::iter())
            .join(
                sea_orm::JoinType::InnerJoin,
                Entity::belongs_to(SongEntity)
                    .from(Column::SongId)
                    .to(SongColumn::FileHash)
                    .into(),
            )
            .order_by_desc(Column::PlayedAt)
            .limit(1)
            .into_model::<SongModel>()
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_last_10_played(
        db: &DatabaseConnection,
    ) -> Result<[SongModel; 10], JudeHarleyError> {
        Entity::find()
            .select_only()
            .columns(SongColumn::iter())
            .join(
                sea_orm::JoinType::InnerJoin,
                Entity::belongs_to(SongEntity)
                    .from(Column::SongId)
                    .to(SongColumn::FileHash)
                    .into(),
            )
            .order_by_desc(Column::PlayedAt)
            .limit(10)
            .into_model::<SongModel>()
            .all(db)
            .await
            .map_err(Into::into)
            .map(|s| s.try_into().unwrap())
    }

    pub async fn count(song: &SongModel, db: &DatabaseConnection) -> Result<i64, JudeHarleyError> {
        Entity::find()
            .select_only()
            .column_as(Column::Id.count(), "count")
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .into_model::<CountQuery>()
            .one(db)
            .await
            .map_err(Into::into)
            .map(|c| c.map(|c| c.count).unwrap_or(0))
    }
}
