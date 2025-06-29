use sea_orm::{FromQueryResult, QueryOrder, QuerySelect, Statement, prelude::*};
use sea_query::IntoCondition;

use crate::{
    dtos::page::{Page, PaginationParams},
    entities,
    repositories::RepositoryError,
};

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
    async fn find_all(
        &self,
        pagination: &PaginationParams,
    ) -> Result<Page<entities::songs::Model>, RepositoryError>;

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
    async fn search(
        &self,
        query: &str,
        pagination: &PaginationParams,
    ) -> Result<Page<entities::songs::Model>, RepositoryError>;

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
        pagination: &PaginationParams,
    ) -> Result<Page<entities::songs::Model>, RepositoryError>;

    /// Count the total number of songs in the database.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while counting the
    /// songs.
    async fn count(&self) -> Result<u64, RepositoryError>;

    /// Find the songs that have been requested recently.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// songs.
    async fn find_recently_requested(
        &self,
        limit: u64,
    ) -> Result<Vec<entities::songs::Model>, RepositoryError>;

    /// Find the songs that have been played recently.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// songs.
    async fn find_recently_played(
        &self,
        pagination: &PaginationParams,
    ) -> Result<Page<entities::songs::Model>, RepositoryError>;
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

    async fn find_all(
        &self,
        pagination: &PaginationParams,
    ) -> Result<Page<entities::songs::Model>, RepositoryError> {
        let paginator = entities::songs::Entity::find().paginate(&self.db, pagination.page_size);

        let page = paginator.fetch_page(pagination.page).await?;
        let total = paginator.num_items_and_pages().await?;

        Ok(Page::new(
            page,
            total.number_of_items,
            pagination.page,
            pagination.page_size,
            total.number_of_pages,
        ))
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

    async fn search(
        &self,
        query: &str,
        pagination: &PaginationParams,
    ) -> Result<Page<entities::songs::Model>, RepositoryError> {
        let paginator = entities::songs::Model::find_by_statement(Statement::from_sql_and_values(
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
        .paginate(&self.db, pagination.page_size);

        let page = paginator.fetch_page(pagination.page).await?;
        let total = paginator.num_items_and_pages().await?;

        Ok(Page::new(
            page,
            total.number_of_items,
            pagination.page,
            pagination.page_size,
            total.number_of_pages,
        ))
    }

    async fn search_favourite_songs(
        &self,
        user_id: i64,
        query: &str,
        pagination: &PaginationParams,
    ) -> Result<Page<entities::songs::Model>, RepositoryError> {
        let paginator = entities::songs::Model::find_by_statement(Statement::from_sql_and_values(
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
        .paginate(&self.db, pagination.page_size);

        let page = paginator.fetch_page(pagination.page).await?;
        let total = paginator.num_items_and_pages().await?;

        Ok(Page::new(
            page,
            total.number_of_items,
            pagination.page,
            pagination.page_size,
            total.number_of_pages,
        ))
    }

    async fn count(&self) -> Result<u64, RepositoryError> {
        entities::songs::Entity::find()
            .count(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn find_recently_requested(
        &self,
        limit: u64,
    ) -> Result<Vec<entities::songs::Model>, RepositoryError> {
        entities::songs::Entity::find()
            .inner_join(entities::song_requests::Entity)
            .order_by_desc(entities::song_requests::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn find_recently_played(
        &self,
        pagination: &PaginationParams,
    ) -> Result<Page<entities::songs::Model>, RepositoryError> {
        let paginator = entities::songs::Entity::find()
            .inner_join(entities::played_songs::Entity)
            .order_by_desc(entities::played_songs::Column::PlayedAt)
            .paginate(&self.db, pagination.page_size);

        let page = paginator.fetch_page(pagination.page).await?;
        let total = paginator.num_items_and_pages().await?;

        Ok(Page::new(
            page,
            total.number_of_items,
            pagination.page,
            pagination.page_size,
            total.number_of_pages,
        ))
    }
}

mod custom_entity {
    use sea_orm::entity::prelude::*;

    use crate::entities;

    #[derive(Clone, Copy, Debug, EnumIter)]
    pub enum SongRelation {
        SongRequests,
        PlayedSongs,
    }

    #[derive(Clone, Copy, Debug, EnumIter)]
    pub enum SongRequestRelation {
        Song,
    }

    #[derive(Clone, Copy, Debug, EnumIter)]
    pub enum PlayedSongRelation {
        Song,
    }

    impl RelationTrait for SongRelation {
        fn def(&self) -> RelationDef {
            match self {
                SongRelation::SongRequests => {
                    entities::song_requests::Entity::belongs_to(entities::songs::Entity)
                        .from(entities::song_requests::Column::SongId)
                        .to(entities::songs::Column::FileHash)
                        .into()
                }
                SongRelation::PlayedSongs => {
                    entities::played_songs::Entity::belongs_to(entities::songs::Entity)
                        .from(entities::played_songs::Column::SongId)
                        .to(entities::songs::Column::FileHash)
                        .into()
                }
            }
        }
    }

    impl RelationTrait for SongRequestRelation {
        fn def(&self) -> RelationDef {
            match self {
                SongRequestRelation::Song => {
                    entities::songs::Entity::has_many(entities::song_requests::Entity).into()
                }
            }
        }
    }

    impl RelationTrait for PlayedSongRelation {
        fn def(&self) -> RelationDef {
            match self {
                PlayedSongRelation::Song => {
                    entities::songs::Entity::has_many(entities::played_songs::Entity).into()
                }
            }
        }
    }

    impl Related<entities::songs::Entity> for entities::song_requests::Entity {
        fn to() -> RelationDef {
            SongRelation::SongRequests.def()
        }
    }

    impl Related<entities::song_requests::Entity> for entities::songs::Entity {
        fn to() -> RelationDef {
            SongRequestRelation::Song.def()
        }
    }

    impl Related<entities::songs::Entity> for entities::played_songs::Entity {
        fn to() -> RelationDef {
            PlayedSongRelation::Song.def()
        }
    }

    impl Related<entities::played_songs::Entity> for entities::songs::Entity {
        fn to() -> RelationDef {
            PlayedSongRelation::Song.def()
        }
    }
}
