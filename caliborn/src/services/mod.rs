use std::{
    fmt::Display,
    sync::{Arc, OnceLock},
};

use cooldowns::CooldownServiceImpl;
use hmac::Hmac;
use sha2::Sha256;
use songs::SongServiceImpl;
use tokio::sync::Mutex;

use crate::{
    DiscordOAuthClient,
    liquidsoap::LiquidsoapClient,
    repositories::RepositoryFactory,
    services::{
        auth::AuthServiceImpl, cans::CansServiceImpl, economy::EconomyServiceImpl,
        users::UserServiceImpl,
    },
};

pub mod auth;
pub mod cans;
pub mod cooldowns;
pub mod economy;
pub mod songs;
pub mod users;

pub struct CachedService<T> {
    cell: Arc<OnceLock<Arc<T>>>,
}

impl<T> CachedService<T> {
    pub fn new() -> Self {
        Self {
            cell: Arc::new(OnceLock::new()),
        }
    }

    pub fn get_or_init<F>(&self, init: F) -> Arc<T>
    where
        F: FnOnce() -> T,
    {
        self.cell.get_or_init(|| Arc::new(init())).clone()
    }
}

impl<T> Clone for CachedService<T> {
    fn clone(&self) -> Self {
        Self {
            cell: Arc::clone(&self.cell),
        }
    }
}

#[derive(Clone, Copy)]
pub struct UserId(u64);

impl From<UserId> for u64 {
    fn from(id: UserId) -> Self {
        id.0
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        UserId(id)
    }
}

impl From<UserId> for i64 {
    fn from(id: UserId) -> Self {
        id.0 as i64
    }
}

impl From<i64> for UserId {
    fn from(id: i64) -> Self {
        UserId(id as u64)
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct ServiceRegistry {
    repository_factory: Arc<dyn RepositoryFactory>,
    jwt_secret: Hmac<Sha256>,
    hmac_secret: Hmac<Sha256>,
    oauth_client: DiscordOAuthClient,
    liquidsoap_client: Arc<Mutex<LiquidsoapClient>>,

    // services
    auth_service: CachedService<AuthServiceImpl>,
    economy_service: CachedService<EconomyServiceImpl>,
    user_service: CachedService<UserServiceImpl>,
    can_service: CachedService<CansServiceImpl>,
    cooldown_service: CachedService<CooldownServiceImpl>,
    song_service: CachedService<SongServiceImpl>,
}

impl ServiceRegistry {
    pub fn new(
        factory: Arc<dyn RepositoryFactory>,
        jwt_secret: Hmac<Sha256>,
        hmac_secret: Hmac<Sha256>,
        oauth_client: DiscordOAuthClient,
        liquidsoap_client: Arc<Mutex<LiquidsoapClient>>,
    ) -> Self {
        Self {
            repository_factory: factory,
            jwt_secret,
            hmac_secret,
            oauth_client,
            auth_service: CachedService::new(),
            economy_service: CachedService::new(),
            user_service: CachedService::new(),
            can_service: CachedService::new(),
            cooldown_service: CachedService::new(),
            song_service: CachedService::new(),
            liquidsoap_client,
        }
    }

    pub fn auth_service(&self) -> Arc<AuthServiceImpl> {
        self.auth_service.get_or_init(|| {
            let user_repo = self.repository_factory.user_repository();
            AuthServiceImpl::new(
                user_repo,
                self.oauth_client.clone(),
                self.jwt_secret.clone(),
                self.hmac_secret.clone(),
            )
        })
    }

    pub fn economy_service(&self) -> Arc<EconomyServiceImpl> {
        self.economy_service.get_or_init(|| {
            let user_repo = self.repository_factory.user_repository();
            let user_service = self.user_service();
            EconomyServiceImpl::new(user_repo, user_service)
        })
    }

    pub fn user_service(&self) -> Arc<UserServiceImpl> {
        self.user_service.get_or_init(|| {
            let user_repo = self.repository_factory.user_repository();
            let cooldown_service = self.cooldown_service();
            UserServiceImpl::new(user_repo, cooldown_service)
        })
    }

    pub fn can_service(&self) -> Arc<CansServiceImpl> {
        self.can_service.get_or_init(|| {
            let can_repo = self.repository_factory.can_repository();
            CansServiceImpl::new(can_repo)
        })
    }

    pub fn cooldown_service(&self) -> Arc<CooldownServiceImpl> {
        self.cooldown_service.get_or_init(|| {
            let cooldown_repo = self.repository_factory.cooldown_repository();
            CooldownServiceImpl::new(cooldown_repo)
        })
    }

    pub fn song_service(&self) -> Arc<SongServiceImpl> {
        self.song_service.get_or_init(|| {
            let song_repo = self.repository_factory.song_repository();
            let user_repo = self.repository_factory.user_repository();
            let song_request_repo = self.repository_factory.song_request_repository();
            let song_history_repo = self.repository_factory.song_history_repository();
            let tag_repo = self.repository_factory.tag_repository();
            let user_service = self.user_service();
            let cooldown_service = self.cooldown_service();

            SongServiceImpl::new(
                song_repo,
                user_repo,
                song_request_repo,
                song_history_repo,
                tag_repo,
                user_service,
                cooldown_service,
                self.liquidsoap_client.clone(),
            )
        })
    }
}
