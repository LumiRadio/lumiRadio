use sea_orm::{ActiveValue, prelude::*};

use crate::{entities, repositories::RepositoryError};

/// A trait representing a repository for song tags.
#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    /// Get all tags for a specific song.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// song tags.
    async fn get_song_tags(
        &self,
        file_hash: &str,
    ) -> Result<Vec<entities::song_tags::Model>, RepositoryError>;

    /// Insert a new tag for a song.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while inserting the
    /// tag.
    async fn insert(
        &self,
        file_hash: &str,
        tag: &str,
        value: &str,
    ) -> Result<entities::song_tags::Model, RepositoryError>;

    /// Insert multiple tags for a song.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while inserting the
    /// tags.
    async fn insert_many(
        &self,
        file_hash: &str,
        tags: &[(&str, &str)],
    ) -> Result<(), RepositoryError>;

    /// Delete all tags for a specific song.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while deleting the
    /// tags.
    async fn delete_by_song(&self, file_hash: &str) -> Result<(), RepositoryError>;

    /// Prune tags for songs that no longer exist.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while pruning the
    /// tags.
    async fn prune(&self) -> Result<(), RepositoryError>;
}

/// A SeaORM implementation of the `TagRepository` trait.
pub struct SeaOrmTagRepository {
    db: DatabaseConnection,
}

impl SeaOrmTagRepository {
    /// Create a new instance of `SeaOrmTagRepository`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a SeaORM database connection.
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl TagRepository for SeaOrmTagRepository {
    async fn get_song_tags(
        &self,
        file_hash: &str,
    ) -> Result<Vec<entities::song_tags::Model>, RepositoryError> {
        entities::song_tags::Entity::find()
            .filter(entities::song_tags::Column::SongId.eq(file_hash))
            .all(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn insert(
        &self,
        file_hash: &str,
        tag: &str,
        value: &str,
    ) -> Result<entities::song_tags::Model, RepositoryError> {
        entities::song_tags::ActiveModel {
            song_id: ActiveValue::set(file_hash.to_string()),
            tag: ActiveValue::set(tag.to_string()),
            value: ActiveValue::set(value.to_string()),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(RepositoryError::from)
    }

    async fn insert_many(
        &self,
        file_hash: &str,
        tags: &[(&str, &str)],
    ) -> Result<(), RepositoryError> {
        if tags.is_empty() {
            return Ok(());
        }

        let to_insert = tags
            .iter()
            .map(|(tag, value)| entities::song_tags::ActiveModel {
                song_id: ActiveValue::set(file_hash.to_string()),
                tag: ActiveValue::set(tag.to_string()),
                value: ActiveValue::set(value.to_string()),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        entities::song_tags::Entity::insert_many(to_insert)
            .exec(&self.db)
            .await
            .map_err(RepositoryError::from)?;

        Ok(())
    }

    async fn delete_by_song(&self, file_hash: &str) -> Result<(), RepositoryError> {
        entities::song_tags::Entity::delete_many()
            .filter(entities::song_tags::Column::SongId.eq(file_hash))
            .exec(&self.db)
            .await
            .map_err(RepositoryError::from)?;

        Ok(())
    }

    async fn prune(&self) -> Result<(), RepositoryError> {
        entities::song_tags::Entity::delete_many()
            .exec(&self.db)
            .await
            .map_err(RepositoryError::from)?;

        Ok(())
    }
}
