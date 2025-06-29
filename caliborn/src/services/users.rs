use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use sea_orm::ActiveValue;

use crate::{
    dtos::{
        error::{PublicError, ToPublicError},
        users::UserDto,
    },
    entities,
    repositories::{RepositoryError, users::UserRepository},
    services::cooldowns::CooldownService,
};

use super::{
    UserId,
    cooldowns::{
        CooldownServiceError, CooldownServiceImpl, UserCooldown, user::UserActivityCooldown,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum UserServiceError {
    #[error("Error while checking activity cooldown")]
    ActivityCooldownError(#[from] CooldownServiceError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl ToPublicError for UserServiceError {
    fn as_public(&self) -> Option<PublicError> {
        match self {
            UserServiceError::ActivityCooldownError(cd) => cd.as_public(),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
pub trait UserService: Send + Sync + 'static {
    async fn create_user(&self, id: UserId) -> Result<UserDto, UserServiceError>;
    async fn get_user(&self, id: UserId) -> Result<UserDto, UserServiceError>;
    async fn update_user_boonbucks(&self, id: UserId, amount: i32) -> Result<(), UserServiceError>;
    async fn update_user_activity_time(&self, id: UserId) -> Result<(), UserServiceError>;
    async fn update_user_activity(&self, id: UserId) -> Result<(), UserServiceError>;
}

pub struct UserServiceImpl {
    user_repo: Box<dyn UserRepository>,

    cooldown_service: Arc<dyn CooldownService>,
}

impl UserServiceImpl {
    pub fn new(repo: Box<dyn UserRepository>, cooldown_service: Arc<dyn CooldownService>) -> Self {
        Self {
            user_repo: repo,
            cooldown_service,
        }
    }
}

#[async_trait::async_trait]
impl UserService for UserServiceImpl {
    /// Creates a new user
    ///
    /// If the user already exists, it will be returned
    async fn create_user(&self, id: UserId) -> Result<UserDto, UserServiceError> {
        let user = match self.user_repo.find_by_id(id.into()).await? {
            Some(user) => user,
            None => self.user_repo.insert(id.into()).await?,
        };

        Ok(user.into())
    }

    /// Gets a user by their ID
    ///
    /// If the user does not exist, it will be created
    async fn get_user(&self, id: UserId) -> Result<UserDto, UserServiceError> {
        self.create_user(id).await
    }

    async fn update_user_boonbucks(&self, id: UserId, amount: i32) -> Result<(), UserServiceError> {
        // ensure user exists
        self.get_user(id).await?;

        self.user_repo
            .update(
                id.into(),
                entities::users::ActiveModel {
                    boonbucks: ActiveValue::set(amount),
                    ..Default::default()
                },
            )
            .await?;

        Ok(())
    }

    async fn update_user_activity_time(&self, id: UserId) -> Result<(), UserServiceError> {
        let user = self.get_user(id).await?;
        let now = Utc::now().naive_utc();
        let time_diff = if let Some(last_message_sent) = user.last_message_sent {
            now - last_message_sent
        } else {
            TimeDelta::zero()
        };

        self.user_repo
            .update(
                id.into(),
                entities::users::ActiveModel {
                    last_message_sent: ActiveValue::set(Some(now)),
                    watched_time: ActiveValue::set(
                        user.watched_time
                            + chrono::Duration::minutes(15).min(time_diff).num_seconds(),
                    ),
                    ..Default::default()
                },
            )
            .await?;

        Ok(())
    }

    async fn update_user_activity(&self, id: UserId) -> Result<(), UserServiceError> {
        self.update_user_activity_time(id).await?;
        let user = self.get_user(id).await?;
        let cooldown = UserActivityCooldown;
        if cooldown.on_cooldown(&self.cooldown_service, id).await? {
            return Ok(());
        }

        self.update_user_boonbucks(id, user.boonbucks + 3).await?;

        Ok(())
    }
}
