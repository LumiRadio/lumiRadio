use sea_orm::{prelude::*, Iterable, QuerySelect, Set};

use crate::custom_entities::songs::{
    Column as SongColumn, Entity as SongEntity, Model as SongModel,
};
use crate::entities::{
    favourite_songs::*,
    users::{Column as UserColumn, Model as UserModel},
};
use crate::prelude::JudeHarleyError;

impl Model {
    pub async fn get_by_user_and_song(
        user: &UserModel,
        song: &SongModel,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::UserId.eq(user.id))
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_by_user(
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<Vec<SongModel>, JudeHarleyError> {
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
            .filter(Column::UserId.eq(user.id))
            .into_model::<SongModel>()
            .all(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_by_song(
        song: &SongModel,
        db: &DatabaseConnection,
    ) -> Result<Vec<UserModel>, JudeHarleyError> {
        Entity::find()
            .select_only()
            .columns(UserColumn::iter())
            .join(sea_orm::JoinType::InnerJoin, Relation::Users.def())
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .into_model::<UserModel>()
            .all(db)
            .await
            .map_err(Into::into)
    }

    pub async fn insert(
        song: &SongModel,
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        ActiveModel {
            user_id: Set(user.id),
            song_id: Set(song.file_hash.clone()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(())
    }

    pub async fn delete_by_user_and_song(
        user: &UserModel,
        song: &SongModel,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user.id))
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .exec(db)
            .await?;

        Ok(())
    }
}
