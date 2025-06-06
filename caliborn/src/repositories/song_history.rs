use sea_orm::{ActiveValue, QueryOrder, QuerySelect, prelude::*};

use crate::{entities, repositories::RepositoryError};

/// A trait representing a repository for song play history.
#[async_trait::async_trait]
pub trait SongHistoryRepository: Send + Sync + 'static {
    /// Record a song play in the history.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while recording the
    /// song play.
    async fn record_play(&self, file_hash: &str) -> Result<(), RepositoryError>;

    /// Get a list of recently played songs.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// recent plays.
    async fn recent_plays(
        &self,
        limit: u64,
    ) -> Result<Vec<entities::played_songs::Model>, RepositoryError>;

    /// Get the play count for a specific song.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while counting the
    /// plays.
    async fn play_count(&self, file_hash: &str) -> Result<u64, RepositoryError>;
}

/// A SeaORM implementation of the `SongHistoryRepository` trait.
pub struct SeaOrmSongHistoryRepository {
    db: DatabaseConnection,
}

impl SeaOrmSongHistoryRepository {
    /// Create a new instance of `SeaOrmSongHistoryRepository`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a SeaORM database connection.
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl SongHistoryRepository for SeaOrmSongHistoryRepository {
    async fn record_play(&self, file_hash: &str) -> Result<(), RepositoryError> {
        entities::played_songs::ActiveModel {
            song_id: ActiveValue::set(file_hash.to_string()),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;

        Ok(())
    }

    async fn recent_plays(
        &self,
        limit: u64,
    ) -> Result<Vec<entities::played_songs::Model>, RepositoryError> {
        entities::played_songs::Entity::find()
            .order_by_desc(entities::played_songs::Column::PlayedAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn play_count(&self, file_hash: &str) -> Result<u64, RepositoryError> {
        entities::played_songs::Entity::find()
            .filter(entities::played_songs::Column::SongId.eq(file_hash))
            .count(&self.db)
            .await
            .map_err(RepositoryError::from)
    }
}
