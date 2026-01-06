use crate::JudeHarleyError;
use crate::custom_entities::songs::Model as SongModel;
use crate::entities::song_tags::*;
use sea_orm::{Set, prelude::*};

pub struct NewTag(pub String, pub String);

impl Model {
    pub async fn insert_many(
        song: &SongModel,
        tags: &[NewTag],
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        Entity::delete_many()
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .exec(db)
            .await?;

        if tags.is_empty() {
            return Ok(());
        }

        Entity::insert_many(tags.iter().map(|t| ActiveModel {
            song_id: Set(song.file_hash.clone()),
            tag: Set(t.0.clone()),
            value: Set(t.1.clone()),
            ..Default::default()
        }))
        .exec(db)
        .await?;

        Ok(())
    }

    pub async fn get_by_song(
        song: &SongModel,
        db: &DatabaseConnection,
    ) -> Result<Vec<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .all(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_tag_for_song(
        song: &SongModel,
        tag: &str,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .filter(Column::Tag.eq(tag))
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_by_song(
        song: &SongModel,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        Entity::delete_many()
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn delete_many(
        songs: &[SongModel],
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        Entity::delete_many()
            .filter(Column::SongId.is_in(songs.iter().map(|s| s.file_hash.clone())))
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn prune(db: &DatabaseConnection) -> Result<(), JudeHarleyError> {
        Entity::delete_many().exec(db).await?;

        Ok(())
    }
}
