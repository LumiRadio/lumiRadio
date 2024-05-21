use chrono::NaiveDateTime;
use sea_orm::{prelude::*, FromQueryResult, QueryOrder, QuerySelect, Set};

use crate::controllers::CountQuery;
use crate::custom_entities::songs::Model as SongModel;
use crate::entities::{song_requests::*, users::Model as UserModel};
use crate::prelude::JudeHarleyError;

#[derive(FromQueryResult)]
struct CreatedAtQuery {
    created_at: NaiveDateTime,
}

impl Model {
    pub async fn insert(
        song: &SongModel,
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        ActiveModel {
            song_id: Set(song.file_hash.clone()),
            user_id: Set(user.id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(())
    }

    pub async fn get_last_requested_for_song(
        song: &SongModel,
        db: &DatabaseConnection,
    ) -> Result<NaiveDateTime, JudeHarleyError> {
        Entity::find()
            .select_only()
            .column(Column::CreatedAt)
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .order_by_desc(Column::CreatedAt)
            .limit(1)
            .into_model::<CreatedAtQuery>()
            .one(db)
            .await
            .map_err(Into::into)
            .map(|c| c.map(|c| c.created_at).unwrap_or_default())
    }

    pub async fn count(song: &SongModel, db: &DatabaseConnection) -> Result<i64, JudeHarleyError> {
        Entity::find()
            .select_only()
            .column_as(Column::SongId.count(), "count")
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .into_model::<CountQuery>()
            .one(db)
            .await
            .map_err(Into::into)
            .map(|c| c.map(|c| c.count).unwrap_or(0))
    }
}
