use std::fmt::Display;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::{
    RepositoryError,
    dtos::error::{PublicError, ToPublicError},
    repositories::cooldowns::CooldownRepository,
};

use super::UserId;

#[derive(Debug, thiserror::Error)]
pub enum CooldownServiceError {
    #[error("Global cooldown `{0}` already exists")]
    GlobalCooldownAlreadyExists(String),
    #[error("User cooldown `{0}` already exists")]
    UserCooldownAlreadyExists(String),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl ToPublicError for CooldownServiceError {
    fn as_public(&self) -> Option<PublicError> {
        None
    }
}

pub struct CooldownService {
    cooldown_repository: Box<dyn CooldownRepository>,
}

impl CooldownService {
    pub fn new(repo: Box<dyn CooldownRepository>) -> Self {
        Self {
            cooldown_repository: repo,
        }
    }

    pub async fn set_global_cooldown(
        &self,
        key: &str,
        duration: chrono::Duration,
    ) -> Result<(), CooldownServiceError> {
        if self.cooldown_repository.get_global(key).await?.is_some() {
            return Err(CooldownServiceError::GlobalCooldownAlreadyExists(
                key.to_string(),
            ));
        }

        let now = chrono::Utc::now();
        self.cooldown_repository
            .create_global(key, (now + duration).naive_utc())
            .await
            .map_err(|e| CooldownServiceError::from(e))
    }

    pub async fn set_user_cooldown(
        &self,
        key: &str,
        user_id: UserId,
        duration: chrono::Duration,
    ) -> Result<(), CooldownServiceError> {
        if self
            .cooldown_repository
            .get(key, user_id.into())
            .await?
            .is_some()
        {
            return Err(CooldownServiceError::UserCooldownAlreadyExists(
                key.to_string(),
            ));
        }

        let now = chrono::Utc::now();
        self.cooldown_repository
            .create_user(user_id.into(), key, (now + duration).naive_utc())
            .await
            .map_err(|e| CooldownServiceError::from(e))
    }

    pub async fn get_user_cooldown(
        &self,
        key: &str,
        user_id: UserId,
    ) -> Result<Option<NaiveDateTime>, CooldownServiceError> {
        let cooldown = self.cooldown_repository.get(key, user_id.into()).await?;
        let expires_at = cooldown.map(|m| m.expires_at);

        Ok(expires_at)
    }

    pub async fn get_global_cooldown(
        &self,
        key: &str,
    ) -> Result<Option<NaiveDateTime>, CooldownServiceError> {
        let cooldown = self.cooldown_repository.get_global(key).await?;
        let expires_at = cooldown.map(|m| m.expires_at);

        Ok(expires_at)
    }

    pub async fn is_on_user_cooldown(
        &self,
        key: &str,
        user_id: UserId,
    ) -> Result<bool, CooldownServiceError> {
        let cooldown = self.cooldown_repository.get(key, user_id.into()).await?;
        let expires_at = cooldown
            .map(|m| m.expires_at)
            .unwrap_or(DateTime::<Utc>::MIN_UTC.naive_utc());

        Ok(expires_at > chrono::Utc::now().naive_utc())
    }

    pub async fn is_on_global_cooldown(&self, key: &str) -> Result<bool, CooldownServiceError> {
        let cooldown = self.cooldown_repository.get_global(key).await?;
        let expires_at = cooldown
            .map(|m| m.expires_at)
            .unwrap_or(DateTime::<Utc>::MIN_UTC.naive_utc());

        Ok(expires_at > chrono::Utc::now().naive_utc())
    }
}

pub trait GlobalCooldown: Display {
    fn duration(&self) -> chrono::Duration;

    async fn set<S: AsRef<CooldownService>>(&self, service: S) -> Result<(), CooldownServiceError> {
        service
            .as_ref()
            .set_global_cooldown(&self.to_string(), self.duration())
            .await
    }

    async fn get<S: AsRef<CooldownService>>(
        &self,
        service: S,
    ) -> Result<Option<NaiveDateTime>, CooldownServiceError> {
        service
            .as_ref()
            .get_global_cooldown(&self.to_string())
            .await
    }

    async fn on_cooldown<S: AsRef<CooldownService>>(
        &self,
        service: S,
    ) -> Result<bool, CooldownServiceError> {
        service
            .as_ref()
            .is_on_global_cooldown(&self.to_string())
            .await
    }
}

pub trait UserCooldown: Display {
    fn duration(&self) -> chrono::Duration;

    async fn set<S: AsRef<CooldownService>>(
        &self,
        service: S,
        user_id: UserId,
    ) -> Result<(), CooldownServiceError> {
        service
            .as_ref()
            .set_user_cooldown(&self.to_string(), user_id, self.duration())
            .await
    }

    async fn get<S: AsRef<CooldownService>>(
        &self,
        service: S,
        user_id: UserId,
    ) -> Result<Option<NaiveDateTime>, CooldownServiceError> {
        service
            .as_ref()
            .get_user_cooldown(&self.to_string(), user_id)
            .await
    }

    async fn on_cooldown<S: AsRef<CooldownService>>(
        &self,
        service: S,
        user_id: UserId,
    ) -> Result<bool, CooldownServiceError> {
        service
            .as_ref()
            .is_on_user_cooldown(&self.to_string(), user_id)
            .await
    }
}

pub mod user {
    use std::fmt::Display;

    use super::{CooldownService, UserCooldown};

    macro_rules! impl_user_cooldown {
        ($self_:ident, $cooldown:ty = $key:expr, $duration:expr) => {
            impl Display for $cooldown {
                fn fmt(&$self_, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", $key)
                }
            }

            impl UserCooldown for $cooldown {
                fn duration(&$self_) -> chrono::Duration {
                    $duration
                }
            }
        };

        ($cooldown:ty = $key:expr, $duration:expr) => {
            impl Display for $cooldown {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", $key)
                }
            }

            impl UserCooldown for $cooldown {
                fn duration(&self) -> chrono::Duration {
                    $duration
                }
            }
        };
    }

    pub struct RollDiceCooldown;
    pub struct SlotCooldown;
    pub struct PvPCooldown;
    pub struct SongRequestCooldown;
    pub struct UserActivityCooldown;

    impl_user_cooldown!(
        RollDiceCooldown = "minigames_roll_dice",
        chrono::Duration::minutes(5)
    );
    impl_user_cooldown!(
        SlotCooldown = "minigames_slots",
        chrono::Duration::minutes(5)
    );
    impl_user_cooldown!(PvPCooldown = "minigames_pvp", chrono::Duration::minutes(5));
    impl_user_cooldown!(
        SongRequestCooldown = "song_request",
        chrono::Duration::minutes(90)
    );
    impl_user_cooldown!(
        UserActivityCooldown = "user_activity",
        chrono::Duration::minutes(5)
    );
}

pub mod global {
    use std::fmt::Display;

    use super::{CooldownService, GlobalCooldown};

    macro_rules! impl_global_cooldown {
        ($self_:ident, $cooldown:ty = $key:expr, $duration:expr) => {
            impl Display for $cooldown {
                fn fmt(&$self_, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", $key)
                }
            }

            impl GlobalCooldown for $cooldown {
                fn duration(&$self_) -> chrono::Duration {
                    $duration
                }
            }
        };

        ($cooldown:ty = $key:expr, $duration:expr) => {
            impl Display for $cooldown {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", $key)
                }
            }

            impl GlobalCooldown for $cooldown {
                fn duration(&self) -> chrono::Duration {
                    $duration
                }
            }
        };
    }

    pub struct SongCooldown {
        file_hash: String,
        song_length: f64,
    }

    impl SongCooldown {
        pub fn new(file_hash: &str, song_length: f64) -> Self {
            Self {
                file_hash: file_hash.to_string(),
                song_length,
            }
        }
    }

    impl_global_cooldown!(
        self,
        SongCooldown = format!("song_request:{}", &self.file_hash),
        match self.song_length {
            0.0..300.0 => chrono::Duration::seconds(1800),
            300.0..600.0 => chrono::Duration::seconds(3600),
            _ => chrono::Duration::seconds(5413),
        }
    );

    pub struct CanCooldown;

    impl_global_cooldown!(CanCooldown = "can", chrono::Duration::seconds(35));
}
