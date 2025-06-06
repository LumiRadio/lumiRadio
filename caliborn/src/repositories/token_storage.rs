use sea_orm::{ActiveValue, prelude::*};

use crate::{entities, repositories::RepositoryError};

/// A trait representing a repository for token storage.
#[async_trait::async_trait]
pub trait TokenStorageRepository: Send + Sync + 'static {
    /// Insert a new token for a user.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while inserting the
    /// token.
    async fn insert(
        &self,
        user_id: i64,
        access_token: &str,
        refresh_token: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<entities::token_storage::Model, RepositoryError>;

    /// Get a token by user ID.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while retrieving the
    /// token.
    async fn get_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Option<entities::token_storage::Model>, RepositoryError>;

    /// Update a token for a user.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while updating the
    /// token.
    async fn update(
        &self,
        user_id: i64,
        access_token: &str,
        refresh_token: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<entities::token_storage::Model, RepositoryError>;

    /// Delete a token for a user.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while deleting the
    /// token.
    async fn delete(&self, user_id: i64) -> Result<(), RepositoryError>;
}

/// A SeaORM implementation of the `TokenStorageRepository` trait.
pub struct SeaOrmTokenStorageRepository {
    db: DatabaseConnection,
}

impl SeaOrmTokenStorageRepository {
    /// Create a new instance of `SeaOrmTokenStorageRepository`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a SeaORM database connection.
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl TokenStorageRepository for SeaOrmTokenStorageRepository {
    async fn insert(
        &self,
        user_id: i64,
        access_token: &str,
        refresh_token: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<entities::token_storage::Model, RepositoryError> {
        entities::token_storage::ActiveModel {
            user_id: ActiveValue::set(user_id),
            access_token: ActiveValue::set(access_token.to_string()),
            refresh_token: ActiveValue::set(refresh_token.to_string()),
            expires_at: ActiveValue::set(expires_at),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(RepositoryError::from)
    }

    async fn get_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Option<entities::token_storage::Model>, RepositoryError> {
        entities::token_storage::Entity::find()
            .filter(entities::token_storage::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(RepositoryError::from)
    }

    async fn update(
        &self,
        user_id: i64,
        access_token: &str,
        refresh_token: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<entities::token_storage::Model, RepositoryError> {
        let results = entities::token_storage::Entity::update_many()
            .filter(entities::token_storage::Column::UserId.eq(user_id))
            .set(entities::token_storage::ActiveModel {
                user_id: ActiveValue::unchanged(user_id),
                access_token: ActiveValue::set(access_token.to_string()),
                refresh_token: ActiveValue::set(refresh_token.to_string()),
                expires_at: ActiveValue::set(expires_at),
                ..Default::default()
            })
            .exec_with_returning(&self.db)
            .await?;

        if results.is_empty() {
            return Err(sea_orm::DbErr::RecordNotUpdated.into());
        }

        Ok(results[0].clone())
    }

    async fn delete(&self, user_id: i64) -> Result<(), RepositoryError> {
        entities::token_storage::Entity::delete_many()
            .filter(entities::token_storage::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
