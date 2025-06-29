use crate::{
    dtos::error::ToPublicError,
    repositories::{RepositoryError, cans::CanRepository},
};

use super::UserId;

#[derive(thiserror::Error, Debug)]
pub enum CansServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl ToPublicError for CansServiceError {
    fn as_public(&self) -> Option<crate::dtos::error::PublicError> {
        match self {
            _ => None,
        }
    }
}

#[async_trait::async_trait]
pub trait CansService: Send + Sync + 'static {
    async fn count(&self) -> Result<u64, CansServiceError>;
    async fn add(&self, user_id: UserId) -> Result<(), CansServiceError>;
}

pub struct CansServiceImpl {
    can_repo: Box<dyn CanRepository>,
}

impl CansServiceImpl {
    pub fn new(repo: Box<dyn CanRepository>) -> Self {
        Self { can_repo: repo }
    }
}

#[async_trait::async_trait]
impl CansService for CansServiceImpl {
    async fn count(&self) -> Result<u64, CansServiceError> {
        let count = self.can_repo.count().await?;

        Ok(count)
    }

    async fn add(&self, user_id: UserId) -> Result<(), CansServiceError> {
        self.can_repo.add(user_id.into(), true).await?;

        Ok(())
    }
}
