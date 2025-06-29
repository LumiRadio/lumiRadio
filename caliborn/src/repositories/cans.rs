use sea_orm::{ActiveValue, FromQueryResult, QuerySelect, prelude::*};

use crate::{entities, repositories::RepositoryError};

/// A trait representing a repository for cans.
#[async_trait::async_trait]
pub trait CanRepository: Send + Sync + 'static {
    /// Add a new can to the database.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while adding the
    /// can.
    async fn add(&self, user_id: i64, legit: bool) -> Result<(), RepositoryError>;

    /// Count the number of cans in the database.
    ///
    /// # Errors
    ///
    /// Returns a `RepositoryError` if something goes wrong while counting the
    /// cans.
    async fn count(&self) -> Result<u64, RepositoryError>;
}

pub struct SeaOrmCanRepository {
    db: DatabaseConnection,
}

impl SeaOrmCanRepository {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl CanRepository for SeaOrmCanRepository {
    async fn add(&self, user_id: i64, legit: bool) -> Result<(), RepositoryError> {
        entities::cans::ActiveModel {
            added_by: ActiveValue::set(user_id),
            legit: ActiveValue::set(legit),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;

        Ok(())
    }

    async fn count(&self) -> Result<u64, RepositoryError> {
        #[derive(FromQueryResult)]
        struct CountResult {
            count: u64,
        }

        entities::cans::Entity::find()
            .select_only()
            .column_as(entities::cans::Column::Id.count(), "count")
            .into_model::<CountResult>()
            .one(&self.db)
            .await
            .map(|count| count.map(|c| c.count).unwrap_or(0))
            .map_err(RepositoryError::from)
    }
}
