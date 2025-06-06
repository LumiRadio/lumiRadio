use sea_orm::{FromQueryResult, Statement, prelude::*};

use crate::{entities, repositories::RepositoryError};

/// A trait representing a repository for songs.
#[async_trait::async_trait]
pub trait SongRepository: Send + Sync + 'static {
    /// Insert a new song into the database.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while inserting the
    /// song.
    async fn insert(
        &self,
        song: entities::songs::ActiveModel,
    ) -> Result<entities::songs::Model, RepositoryError>;

    /// Delete a song by its file path.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while deleting the
    /// song.
    async fn delete(&self, file_path: &str) -> Result<(), RepositoryError>;

    /// Delete a song by its file hash.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while deleting the
    /// song.
    async fn delete_by_hash(&self, file_hash: &str) -> Result<(), RepositoryError>;

    /// Prune songs that no longer exist on disk.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while pruning the
    /// songs.
    async fn prune(&self) -> Result<(), RepositoryError>;

    /// Find all songs in the database.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// songs.
    async fn find_all(&self) -> Result<Vec<entities::songs::Model>, RepositoryError>;

    /// Find a song by its file path.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// song.
    async fn find_by_path(
        &self,
        file_path: &str,
    ) -> Result<Option<entities::songs::Model>, RepositoryError>;

    /// Find a song by its file hash.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// song.
    async fn find_by_hash(
        &self,
        file_hash: &str,
    ) -> Result<Option<entities::songs::Model>, RepositoryError>;

    /// Search for songs matching a query string.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while searching for
    /// songs.
    async fn search(&self, query: &str) -> Result<Vec<entities::songs::Model>, RepositoryError>;

    /// Search for favourite songs of a user matching a query string.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while searching for
    /// favourite songs.
    async fn search_favourite_songs(
        &self,
        user_id: i64,
        query: &str,
    ) -> Result<Vec<entities::songs::Model>, RepositoryError>;

    /// Count the total number of songs in the database.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while counting the
    /// songs.
    async fn count(&self) -> Result<u64, RepositoryError>;
}

/// A SeaORM implementation of the `SongRepository` trait.
pub struct SeaOrmSongRepository {
    db: DatabaseConnection,
}

impl SeaOrmSongRepository {
    /// Create a new instance of `SeaOrmSongRepository`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a SeaORM database connection.
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl SongRepository for SeaOrmSongRepository {
    async fn insert(
        &self,
        song: entities::songs::ActiveModel,
    ) -> Result<entities::songs::Model, RepositoryError> {
        song.insert(&self.db).await.map_err(RepositoryError::from)
    }

    async fn delete(&self, file_path: &str) -> Result<(), RepositoryError> {
        entities::songs::Entity::delete_many()
            .filter(entities::songs::Column::FilePath.eq(file_path))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn delete_by_hash(&self, file_hash: &str) -> Result<(), RepositoryError> {
        entities::songs::Entity::delete_many()
            .filter(entities::songs::Column::FileHash.eq(file_hash))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn prune(&self) -> Result<(), RepositoryError> {
        entities::songs::Entity::delete_many()
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<entities::songs::Model>, RepositoryError> {
        entities::songs::Entity::find()
            .all(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn find_by_path(
        &self,
        file_path: &str,
    ) -> Result<Option<entities::songs::Model>, RepositoryError> {
        entities::songs::Entity::find_by_id(file_path)
            .one(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn find_by_hash(
        &self,
        file_hash: &str,
    ) -> Result<Option<entities::songs::Model>, RepositoryError> {
        entities::songs::Entity::find()
            .filter(entities::songs::Column::FileHash.eq(file_hash))
            .one(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn search(&self, query: &str) -> Result<Vec<entities::songs::Model>, RepositoryError> {
        entities::songs::Model::find_by_statement(Statement::from_sql_and_values(
            self.db.get_database_backend(),
            r#"
            WITH search AS (
                SELECT to_tsquery(string_agg(lexeme || ':*', ' & ' ORDER BY positions)) AS query
                FROM unnest(to_tsvector($1))
            )
            SELECT songs.*
            FROM songs, search
            WHERE tsvector @@ query
            "#,
            [query.into()],
        ))
        .all(&self.db)
        .await
        .map_err(RepositoryError::from)
    }

    async fn search_favourite_songs(
        &self,
        user_id: i64,
        query: &str,
    ) -> Result<Vec<entities::songs::Model>, RepositoryError> {
        entities::songs::Model::find_by_statement(Statement::from_sql_and_values(
            self.db.get_database_backend(),
            r#"
            WITH search AS (
                SELECT to_tsquery(string_agg(lexeme || ':*', ' & ' ORDER BY positions)) AS query
                FROM unnest(to_tsvector($1))
            )
            SELECT songs.*
            FROM search, favourite_songs
            JOIN songs ON favourite_songs.song_id = songs.file_hash
            WHERE favourite_songs.user_id = $2 AND tsvector @@ query
            "#,
            [query.into(), user_id.into()],
        ))
        .all(&self.db)
        .await
        .map_err(RepositoryError::from)
    }

    async fn count(&self) -> Result<u64, RepositoryError> {
        entities::songs::Entity::find()
            .count(&self.db)
            .await
            .map_err(RepositoryError::from)
    }
}
