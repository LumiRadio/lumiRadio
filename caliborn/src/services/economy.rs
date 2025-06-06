use std::sync::Arc;

use reqwest::StatusCode;

use crate::{
    dtos::error::{PublicError, ToPublicError},
    repositories::{RepositoryError, users::UserRepository},
    services::users::{UserService, UserServiceError},
};

use super::UserId;

#[derive(thiserror::Error, Debug)]
pub enum EconomyServiceError {
    #[error("Target user not found")]
    TargetUserNotFound,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    UserService(#[from] UserServiceError),
}

impl ToPublicError for EconomyServiceError {
    fn as_public(&self) -> Option<PublicError> {
        match self {
            EconomyServiceError::TargetUserNotFound => Some(PublicError::new(
                "user-not-found",
                "The target user was not found.",
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
            EconomyServiceError::InvalidAmount => Some(PublicError::new(
                "invalid-amount",
                "The amount provided is invalid.",
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
            EconomyServiceError::InsufficientFunds => Some(PublicError::new(
                "insufficient-funds",
                "The user does not have enough funds.",
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
            EconomyServiceError::UserService(e) => e.as_public(),
            _ => None,
        }
    }
}

pub struct EconomyService {
    user_repo: Box<dyn UserRepository>,
    user_service: Arc<UserService>,
}

impl EconomyService {
    pub fn new(repo: Box<dyn UserRepository>, user_service: Arc<UserService>) -> Self {
        Self {
            user_repo: repo,
            user_service,
        }
    }

    pub async fn get_balance(&self, id: UserId) -> Result<i32, EconomyServiceError> {
        let user = self.user_service.get_user(id).await?;

        Ok(user.boonbucks)
    }

    pub async fn add_boonbucks(&self, id: UserId, amount: i32) -> Result<(), EconomyServiceError> {
        let user = self.user_service.get_user(id).await?;

        self.user_service
            .update_user_boonbucks(id, user.boonbucks + amount)
            .await?;

        Ok(())
    }

    pub async fn remove_boonbucks(
        &self,
        id: UserId,
        amount: i32,
    ) -> Result<(), EconomyServiceError> {
        let user = self.user_service.get_user(id).await?;

        self.user_service
            .update_user_boonbucks(id, user.boonbucks - amount)
            .await?;

        Ok(())
    }

    pub async fn transfer_boonbucks(
        &self,
        from_id: UserId,
        to_id: UserId,
        amount: i32,
    ) -> Result<(), EconomyServiceError> {
        if amount <= 0 {
            return Err(EconomyServiceError::InvalidAmount);
        }

        let balance_from = self.get_balance(from_id).await?;
        if balance_from < amount {
            return Err(EconomyServiceError::InsufficientFunds);
        }

        self.user_repo
            .transfer_boonbucks(from_id.into(), to_id.into(), amount)
            .await?;

        Ok(())
    }
}
