use cooldowns::{CooldownRepository, SeaOrmCooldownRepository};
use sea_orm::DatabaseConnection;

use crate::repositories::{
    cans::{CanRepository, SeaOrmCanRepository},
    server_channel_config::{SeaOrmServerChannelConfigRepository, ServerChannelConfigRepository},
    server_config::{SeaOrmServerConfigRepository, ServerConfigRepository},
    song_history::{SeaOrmSongHistoryRepository, SongHistoryRepository},
    song_requests::{SeaOrmSongRequestRepository, SongRequestRepository},
    songs::{SeaOrmSongRepository, SongRepository},
    tags::{SeaOrmTagRepository, TagRepository},
    // Currently not in use
    // token_storage::{SeaOrmTokenStorageRepository, TokenStorageRepository},
    users::{SeaOrmUserRepository, UserRepository},
};

pub mod cans;
pub mod cooldowns;
pub mod server_channel_config;
pub mod server_config;
pub mod song_history;
pub mod song_requests;
pub mod songs;
pub mod tags;
// Currently not in use, the frontend and bot are supposed to update user data
// pub mod token_storage;
pub mod users;

pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// An enum representing possible errors that can occur when interacting with a repository.
#[derive(thiserror::Error, Debug)]
pub enum RepositoryError {
    /// An error occurred while interacting with the repository. This IS a bug.
    #[error(transparent)]
    Unexpected(#[from] DynError),
}

impl From<sea_orm::DbErr> for RepositoryError {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::Unexpected(anyhow::Error::new(value).context("sea-orm failure").into())
    }
}

impl<E: std::error::Error + Send + Sync + 'static> From<sea_orm::TransactionError<E>>
    for RepositoryError
{
    fn from(value: sea_orm::TransactionError<E>) -> Self {
        Self::Unexpected(
            anyhow::Error::new(value)
                .context("sea-orm transaction failure")
                .into(),
        )
    }
}

/// A factory trait for creating repository instances.
///
/// This trait provides methods to create different types of repositories
/// that are used throughout the application. Implementations of this trait
/// are responsible for creating and configuring repository instances with
/// the appropriate database connections or other dependencies.
pub trait RepositoryFactory: Send + Sync + 'static {
    /// Creates a new instance of a `CanRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `CanRepository` trait.
    fn can_repository(&self) -> Box<dyn CanRepository>;

    /// Creates a new instance of a `ServerChannelConfigRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `ServerChannelConfigRepository` trait.
    fn server_channel_config_repository(&self) -> Box<dyn ServerChannelConfigRepository>;

    /// Creates a new instance of a `SongRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `SongRepository` trait.
    fn song_repository(&self) -> Box<dyn SongRepository>;

    /// Creates a new instance of a `SongHistoryRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `SongHistoryRepository` trait.
    fn song_history_repository(&self) -> Box<dyn SongHistoryRepository>;

    /// Creates a new instance of a `SongRequestRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `SongRequestRepository` trait.
    fn song_request_repository(&self) -> Box<dyn SongRequestRepository>;

    /// Creates a new instance of a `TagRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `TagRepository` trait.
    fn tag_repository(&self) -> Box<dyn TagRepository>;

    /// Creates a new instance of a `TokenStorageRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `TokenStorageRepository` trait.
    // Currently not in use
    // fn token_storage_repository(&self) -> Box<dyn TokenStorageRepository>;

    /// Creates a new instance of a `UserRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `UserRepository` trait.
    fn user_repository(&self) -> Box<dyn UserRepository>;

    /// Creates a new instance of a `ServerConfigRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `ServerConfigRepository` trait.
    fn server_config_repository(&self) -> Box<dyn ServerConfigRepository>;

    /// Creates a new instance of a `CooldownRepository`.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing the `CooldownRepository` trait.
    fn cooldown_repository(&self) -> Box<dyn CooldownRepository>;
}

/// A SeaORM implementation of the `RepositoryFactory` trait.
///
/// This struct holds a database connection that is used to create
/// SeaORM-based repository implementations.
pub struct SeaOrmRepositoryFactory {
    /// The database connection used by all repositories created by this factory.
    db: DatabaseConnection,
}

impl SeaOrmRepositoryFactory {
    /// Creates a new instance of `SeaOrmRepositoryFactory`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a SeaORM database connection that will be cloned and stored.
    ///
    /// # Returns
    ///
    /// A new instance of `SeaOrmRepositoryFactory`.
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl RepositoryFactory for SeaOrmRepositoryFactory {
    fn can_repository(&self) -> Box<dyn CanRepository> {
        Box::new(SeaOrmCanRepository::new(&self.db))
    }

    fn server_channel_config_repository(&self) -> Box<dyn ServerChannelConfigRepository> {
        Box::new(SeaOrmServerChannelConfigRepository::new(&self.db))
    }

    fn song_repository(&self) -> Box<dyn SongRepository> {
        Box::new(SeaOrmSongRepository::new(&self.db))
    }

    fn song_history_repository(&self) -> Box<dyn SongHistoryRepository> {
        Box::new(SeaOrmSongHistoryRepository::new(&self.db))
    }

    fn song_request_repository(&self) -> Box<dyn SongRequestRepository> {
        Box::new(SeaOrmSongRequestRepository::new(&self.db))
    }

    fn tag_repository(&self) -> Box<dyn TagRepository> {
        Box::new(SeaOrmTagRepository::new(&self.db))
    }

    // Currently not in use
    // fn token_storage_repository(&self) -> Box<dyn TokenStorageRepository> {
    //     Box::new(SeaOrmTokenStorageRepository::new(&self.db))
    // }

    fn user_repository(&self) -> Box<dyn UserRepository> {
        Box::new(SeaOrmUserRepository::new(&self.db))
    }

    fn server_config_repository(&self) -> Box<dyn ServerConfigRepository> {
        Box::new(SeaOrmServerConfigRepository::new(&self.db))
    }

    fn cooldown_repository(&self) -> Box<dyn CooldownRepository> {
        Box::new(SeaOrmCooldownRepository::new(&self.db))
    }
}
