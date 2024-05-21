use std::path::Path;

use sea_orm::{prelude::*, FromQueryResult, QuerySelect, Set, Statement};

use crate::entities::{
    favourite_songs::Model as FavouriteSongModel, played_songs::Model as PlayedModel,
    song_requests::Model as RequestModel, song_tags::Model as TagsModel, users::Model as UserModel,
};
use crate::{custom_entities::songs::*, JudeHarleyError};

pub struct InsertParams {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub file_path: String,
    pub file_hash: String,
    pub duration: f64,
    pub bitrate: i32,
}

#[derive(FromQueryResult)]
struct PathQuery {
    file_path: String,
}

impl Model {
    pub async fn insert(
        params: InsertParams,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        ActiveModel {
            file_path: Set(params.file_path),
            file_hash: Set(params.file_hash),
            title: Set(params.title),
            artist: Set(params.artist),
            album: Set(params.album),
            duration: Set(params.duration),
            bitrate: Set(params.bitrate),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(Into::into)
    }

    pub async fn delete(&self, db: &DatabaseConnection) -> Result<(), JudeHarleyError> {
        TagsModel::delete_by_song(self, db).await?;
        Entity::delete_by_id(&self.file_path).exec(db).await?;

        Ok(())
    }

    pub async fn delete_many(
        songs: &[Model],
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        TagsModel::delete_many(songs, db).await?;

        Entity::delete_many()
            .filter(Column::FileHash.is_in(songs.iter().map(|s| s.file_hash.clone())))
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn delete_by_path(
        path: &Path,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        let path = path.display().to_string();

        let songs = Entity::find()
            .filter(Column::FilePath.eq(path.to_string()))
            .all(db)
            .await?;

        Self::delete_many(&songs, db).await?;

        Ok(())
    }

    pub async fn prune(db: &DatabaseConnection) -> Result<(), JudeHarleyError> {
        TagsModel::prune(db).await?;
        Entity::delete_many().exec(db).await?;
        Ok(())
    }

    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Self>, JudeHarleyError> {
        Entity::find().all(db).await.map_err(Into::into)
    }

    pub async fn get(
        file_path: &str,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find_by_id(file_path)
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_by_hash(
        file_hash: &str,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::FileHash.eq(file_hash))
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_by_directory(
        directory: &Path,
        db: &DatabaseConnection,
    ) -> Result<Vec<Self>, JudeHarleyError> {
        if directory.is_file() {
            return Ok(vec![]);
        }

        let directory = directory.display().to_string();
        Entity::find()
            .filter(Column::FilePath.like(format!("{}%", directory)))
            .all(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_all_paths(db: &DatabaseConnection) -> Result<Vec<String>, JudeHarleyError> {
        Entity::find()
            .select_only()
            .column(Column::FilePath)
            .into_model::<PathQuery>()
            .all(db)
            .await
            .map_err(Into::into)
            .map(|p| p.into_iter().map(|q| q.file_path).collect())
    }

    pub async fn search(
        query: &str,
        db: &DatabaseConnection,
    ) -> Result<Vec<Self>, JudeHarleyError> {
        Model::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH search AS (
                SELECT to_tsquery(string_agg(lexeme || ':*', ' & ' ORDER BY positions)) AS query
                FROM unnest(to_tsvector($1))
            )
            SELECT title, artist, album, file_path, duration, file_hash, bitrate
            FROM songs, search
            WHERE tsvector @@ query
            "#,
            [query.into()],
        ))
        .all(db)
        .await
        .map_err(Into::into)
    }

    pub async fn search_favourited_songs(
        query: &str,
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<Vec<Self>, JudeHarleyError> {
        Model::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH search AS (
                SELECT to_tsquery(string_agg(lexeme || ':*', ' & ' ORDER BY positions)) AS query
                FROM unnest(to_tsvector($1))
            )
            SELECT title, artist, album, file_path, duration, file_hash, bitrate
            FROM search, favourite_songs
            JOIN songs ON favourite_songs.song_id = songs.file_hash
            WHERE favourite_songs.user_id = $2 AND tsvector @@ query
            "#,
            [query.into(), user.id.into()],
        ))
        .all(db)
        .await
        .map_err(Into::into)
    }

    pub async fn tags(&self, db: &DatabaseConnection) -> Result<Vec<TagsModel>, JudeHarleyError> {
        TagsModel::get_by_song(self, db).await
    }

    pub async fn tag(
        &self,
        tag: &str,
        db: &DatabaseConnection,
    ) -> Result<Option<String>, JudeHarleyError> {
        TagsModel::get_tag_for_song(self, tag, db)
            .await
            .map(|t| t.map(|t| t.value))
    }

    pub async fn played(&self, db: &DatabaseConnection) -> Result<i64, JudeHarleyError> {
        PlayedModel::count(self, db).await
    }

    pub async fn requested(&self, db: &DatabaseConnection) -> Result<i64, JudeHarleyError> {
        RequestModel::count(self, db).await
    }

    pub async fn request(
        &self,
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        RequestModel::insert(self, user, db).await
    }

    pub async fn last_10_songs(db: &DatabaseConnection) -> Result<[Self; 10], JudeHarleyError> {
        PlayedModel::get_last_10_played(db).await
    }

    pub async fn last_played(db: &DatabaseConnection) -> Result<Option<Self>, JudeHarleyError> {
        PlayedModel::get_last_played(db).await
    }

    pub async fn is_on_cooldown(&self, db: &DatabaseConnection) -> Result<bool, JudeHarleyError> {
        let last_requested = RequestModel::get_last_requested_for_song(self, db).await?;

        let cooldown_time = if self.duration < 300.0 {
            chrono::Duration::seconds(1800)
        } else if self.duration < 600.0 {
            chrono::Duration::seconds(3600)
        } else {
            chrono::Duration::seconds(5413)
        };

        let over = last_requested + cooldown_time;

        Ok(over > chrono::Utc::now().naive_utc())
    }

    pub async fn list_favouritees(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<UserModel>, JudeHarleyError> {
        FavouriteSongModel::get_by_song(self, db).await
    }
}
