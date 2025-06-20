use chrono::NaiveDateTime;
use sea_orm::{ActiveValue, QueryOrder, QuerySelect, prelude::*};

use crate::{entities, repositories::RepositoryError};

/// A trait representing a repository for song requests.
#[async_trait::async_trait]
pub trait SongRequestRepository: Send + Sync + 'static {
    /// Request a song to be played.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while recording the
    /// song request.
    async fn request_song(&self, file_hash: &str, user_id: i64) -> Result<(), RepositoryError>;

    /// Get a list of recent song requests.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// recent requests.
    async fn recent_requests(
        &self,
        limit: u64,
    ) -> Result<Vec<entities::song_requests::Model>, RepositoryError>;

    /// Get the timestamp of the last request for a specific song.
    /// Will return `None` if no requests have been made for the song.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// timestamp.
    async fn last_request_timestamp(
        &self,
        file_hash: &str,
    ) -> Result<Option<NaiveDateTime>, RepositoryError>;

    /// Count the number of requests for a specific song.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while counting the
    /// requests.
    async fn count(&self, file_hash: &str) -> Result<u64, RepositoryError>;

    /// Count the total number of song requests.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while counting the
    /// total requests.
    async fn total_requests(&self) -> Result<u64, RepositoryError>;
}

/// A SeaORM implementation of the `SongRequestRepository` trait.
pub struct SeaOrmSongRequestRepository {
    db: DatabaseConnection,
}

impl SeaOrmSongRequestRepository {
    /// Create a new instance of `SeaOrmSongRequestRepository`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a SeaORM database connection.
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl SongRequestRepository for SeaOrmSongRequestRepository {
    async fn request_song(&self, file_hash: &str, user_id: i64) -> Result<(), RepositoryError> {
        entities::song_requests::ActiveModel {
            song_id: ActiveValue::set(file_hash.to_string()),
            user_id: ActiveValue::set(user_id),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;

        Ok(())
    }

    async fn recent_requests(
        &self,
        limit: u64,
    ) -> Result<Vec<entities::song_requests::Model>, RepositoryError> {
        entities::song_requests::Entity::find()
            .order_by_desc(entities::song_requests::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn count(&self, file_hash: &str) -> Result<u64, RepositoryError> {
        entities::song_requests::Entity::find()
            .filter(entities::song_requests::Column::SongId.eq(file_hash))
            .count(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn total_requests(&self) -> Result<u64, RepositoryError> {
        entities::song_requests::Entity::find()
            .count(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn last_request_timestamp(
        &self,
        file_hash: &str,
    ) -> Result<Option<NaiveDateTime>, RepositoryError> {
        entities::song_requests::Entity::find()
            .filter(entities::song_requests::Column::SongId.eq(file_hash))
            .order_by_desc(entities::song_requests::Column::CreatedAt)
            .limit(1)
            .one(&self.db)
            .await
            .map_err(RepositoryError::from)
            .map(|model| model.map(|model| model.created_at))
    }
}
