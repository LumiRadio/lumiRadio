use std::{fmt::Display, str::FromStr};

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::entities;

use super::RepositoryError;

#[derive(Debug)]
pub struct UnknownCooldownScope(String);

impl Display for UnknownCooldownScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown cooldown scope: {}", self.0)
    }
}

impl std::error::Error for UnknownCooldownScope {}

/// A cooldown scope
pub enum CooldownScope {
    User,
    Global,
}

impl Display for CooldownScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CooldownScope::User => write!(f, "User"),
            CooldownScope::Global => write!(f, "Global"),
        }
    }
}

impl FromStr for CooldownScope {
    type Err = UnknownCooldownScope;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "User" => Ok(CooldownScope::User),
            "Global" => Ok(CooldownScope::Global),
            _ => Err(UnknownCooldownScope(s.to_string())),
        }
    }
}

/// A trait representing a repository for managing cooldowns.
#[async_trait::async_trait]
pub trait CooldownRepository: Send + Sync + 'static {
    /// Retrieves a user cooldown by its key.
    async fn get(
        &self,
        key: &str,
        user_id: i64,
    ) -> Result<Option<entities::cooldown::Model>, RepositoryError>;

    /// Retrieves a global cooldown by its key.
    async fn get_global(
        &self,
        key: &str,
    ) -> Result<Option<entities::cooldown::Model>, RepositoryError>;

    /// Retrieves all cooldowns by their key and, optionally, their scope.
    async fn get_all(
        &self,
        key: &str,
        scope: Option<CooldownScope>,
    ) -> Result<Vec<entities::cooldown::Model>, RepositoryError>;

    /// Creates a new user cooldown.
    async fn create_user(
        &self,
        user_id: i64,
        key: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<(), RepositoryError>;

    /// Creates a new global cooldown.
    async fn create_global(
        &self,
        key: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<(), RepositoryError>;

    /// Deletes a user cooldown by its key.
    async fn delete_user(&self, user_id: i64, key: &str) -> Result<(), RepositoryError>;

    /// Deletes a global cooldown by its key.
    async fn delete_global(&self, key: &str) -> Result<(), RepositoryError>;
}

pub struct SeaOrmCooldownRepository {
    db: DatabaseConnection,
}

impl SeaOrmCooldownRepository {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl CooldownRepository for SeaOrmCooldownRepository {
    async fn get(
        &self,
        key: &str,
        user_id: i64,
    ) -> Result<Option<entities::cooldown::Model>, RepositoryError> {
        entities::cooldown::Entity::find()
            .filter(entities::cooldown::Column::UserId.eq(user_id))
            .filter(entities::cooldown::Column::Scope.eq(CooldownScope::User.to_string()))
            .filter(entities::cooldown::Column::Key.eq(key))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::from(e))
    }

    async fn get_global(
        &self,
        key: &str,
    ) -> Result<Option<entities::cooldown::Model>, RepositoryError> {
        entities::cooldown::Entity::find()
            .filter(entities::cooldown::Column::Scope.eq(CooldownScope::Global.to_string()))
            .filter(entities::cooldown::Column::Key.eq(key))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::from(e))
    }

    async fn get_all(
        &self,
        key: &str,
        scope: Option<CooldownScope>,
    ) -> Result<Vec<entities::cooldown::Model>, RepositoryError> {
        let mut query =
            entities::cooldown::Entity::find().filter(entities::cooldown::Column::Key.eq(key));

        if let Some(scope) = scope {
            query = query.filter(entities::cooldown::Column::Scope.eq(scope.to_string()));
        }

        query
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::from(e))
    }

    async fn create_user(
        &self,
        user_id: i64,
        key: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<(), RepositoryError> {
        entities::cooldown::ActiveModel {
            scope: ActiveValue::set(CooldownScope::User.to_string()),
            user_id: ActiveValue::Set(Some(user_id)),
            key: ActiveValue::Set(key.to_string()),
            expires_at: ActiveValue::Set(expires_at),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;

        Ok(())
    }

    async fn create_global(
        &self,
        key: &str,
        expires_at: chrono::NaiveDateTime,
    ) -> Result<(), RepositoryError> {
        entities::cooldown::ActiveModel {
            scope: ActiveValue::set(CooldownScope::Global.to_string()),
            key: ActiveValue::Set(key.to_string()),
            expires_at: ActiveValue::Set(expires_at),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;

        Ok(())
    }

    async fn delete_user(&self, user_id: i64, key: &str) -> Result<(), RepositoryError> {
        entities::cooldown::Entity::delete_many()
            .filter(entities::cooldown::Column::UserId.eq(user_id))
            .filter(entities::cooldown::Column::Scope.eq(CooldownScope::User.to_string()))
            .filter(entities::cooldown::Column::Key.eq(key))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn delete_global(&self, key: &str) -> Result<(), RepositoryError> {
        entities::cooldown::Entity::delete_many()
            .filter(entities::cooldown::Column::Scope.eq(CooldownScope::Global.to_string()))
            .filter(entities::cooldown::Column::Key.eq(key))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
