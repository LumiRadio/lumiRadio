use sea_orm::{ActiveValue, prelude::*};

use crate::{entities, repositories::RepositoryError};

/// A trait representing a repository for server channel configurations.
#[async_trait::async_trait]
pub trait ServerChannelConfigRepository: Send + Sync + 'static {
    /// Find a server channel configuration by its ID.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// server channel configuration.
    async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<entities::server_channel_config::Model>, RepositoryError>;

    /// Insert a new server channel configuration into the database.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while inserting the
    /// server channel configuration.
    async fn insert(
        &self,
        id: i64,
        server_id: i64,
    ) -> Result<entities::server_channel_config::Model, RepositoryError>;

    /// Update an existing server channel configuration.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while updating the
    /// server channel configuration.
    async fn update(
        &self,
        id: i64,
        params: entities::server_channel_config::ActiveModel,
    ) -> Result<entities::server_channel_config::Model, RepositoryError>;

    /// Find all channels with hydration reminders enabled.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// hydration channels.
    async fn find_hydration_channels(
        &self,
    ) -> Result<Vec<entities::server_channel_config::Model>, RepositoryError>;
}

/// A SeaORM implementation of the `ServerChannelConfigRepository` trait.
pub struct SeaOrmServerChannelConfigRepository {
    db: DatabaseConnection,
}

impl SeaOrmServerChannelConfigRepository {
    /// Create a new instance of `SeaOrmServerChannelConfigRepository`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a SeaORM database connection.
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl ServerChannelConfigRepository for SeaOrmServerChannelConfigRepository {
    async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<entities::server_channel_config::Model>, RepositoryError> {
        entities::server_channel_config::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn insert(
        &self,
        id: i64,
        server_id: i64,
    ) -> Result<entities::server_channel_config::Model, RepositoryError> {
        entities::server_channel_config::ActiveModel {
            id: ActiveValue::set(id),
            server_id: ActiveValue::set(server_id),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(RepositoryError::from)
    }

    async fn update(
        &self,
        id: i64,
        mut params: entities::server_channel_config::ActiveModel,
    ) -> Result<entities::server_channel_config::Model, RepositoryError> {
        params.id = ActiveValue::unchanged(id);

        entities::server_channel_config::Entity::update(params)
            .filter(entities::server_channel_config::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn find_hydration_channels(
        &self,
    ) -> Result<Vec<entities::server_channel_config::Model>, RepositoryError> {
        entities::server_channel_config::Entity::find()
            .filter(entities::server_channel_config::Column::HydrationReminder.eq(true))
            .all(&self.db)
            .await
            .map_err(RepositoryError::from)
    }
}
