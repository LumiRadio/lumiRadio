use sea_orm::{ActiveValue, prelude::*};

use crate::{entities, repositories::RepositoryError};

/// A trait representing a repository for server configurations.
#[async_trait::async_trait]
pub trait ServerConfigRepository: Send + Sync + 'static {
    /// Find a server configuration by its ID.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// server configuration.
    async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<entities::server_config::Model>, RepositoryError>;

    /// Insert a new server configuration into the database.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while inserting the
    /// server configuration.
    async fn insert(&self, id: i64) -> Result<entities::server_config::Model, RepositoryError>;

    /// Update an existing server configuration.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while updating the
    /// server configuration.
    async fn update(
        &self,
        id: i64,
        params: entities::server_config::ActiveModel,
    ) -> Result<entities::server_config::Model, RepositoryError>;
}

/// A SeaORM implementation of the `ServerConfigRepository` trait.
pub struct SeaOrmServerConfigRepository {
    db: DatabaseConnection,
}

impl SeaOrmServerConfigRepository {
    /// Create a new instance of `SeaOrmServerConfigRepository`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a SeaORM database connection.
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl ServerConfigRepository for SeaOrmServerConfigRepository {
    async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<entities::server_config::Model>, RepositoryError> {
        entities::server_config::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn insert(&self, id: i64) -> Result<entities::server_config::Model, RepositoryError> {
        let config = entities::server_config::ActiveModel {
            id: ActiveValue::set(id),
            dice_roll: ActiveValue::Set(111),
            slot_jackpot: ActiveValue::Set(0),
            ..Default::default()
        };

        config.insert(&self.db).await.map_err(RepositoryError::from)
    }

    async fn update(
        &self,
        id: i64,
        mut params: entities::server_config::ActiveModel,
    ) -> Result<entities::server_config::Model, RepositoryError> {
        params.id = ActiveValue::unchanged(id);

        entities::server_config::Entity::update(params)
            .filter(entities::server_config::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(RepositoryError::from)
    }
}
