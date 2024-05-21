use crate::custom_entities::songs::Model as SongModel;
use crate::entities::song_tags::*;
use crate::JudeHarleyError;
use sea_orm::{prelude::*, Set};

pub struct InsertParams {
    pub tag: String,
    pub value: String,
}

impl Model {
    pub async fn insert_many(
        song: &SongModel,
        tags: &[InsertParams],
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        Entity::delete_many()
            .filter(Column::SongId.eq(song.file_hash.clone()))
            .exec(db)
            .await?;

        Entity::insert_many(
            tags.iter()
                .map(|t| ActiveModel {
                    song_id: Set(song.file_hash.clone()),
                    tag: Set(t.tag.clone()),
                    value: Set(t.value.clone()),
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
        )
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
