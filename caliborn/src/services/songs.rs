use std::sync::Arc;

use reqwest::{StatusCode, header::RETRY_AFTER};
use tokio::sync::Mutex;

use crate::{
    RepositoryError,
    dtos::{
        error::{PublicError, ToPublicError},
        page::{Page, PaginationParams},
        songs::{CooldownInfo, SongDto, SongWithCooldownInfo},
    },
    liquidsoap::{LiquidsoapClient, LiquidsoapError, QueueItem},
    repositories::{
        song_history::SongHistoryRepository, song_requests::SongRequestRepository,
        songs::SongRepository, tags::TagRepository, users::UserRepository,
    },
    services::{cooldowns::CooldownService, users::UserService},
};

use super::{
    UserId,
    cooldowns::{
        CooldownServiceError, CooldownServiceImpl, GlobalCooldown, UserCooldown,
        global::SongCooldown, user::SongRequestCooldown,
    },
    users::{UserServiceError, UserServiceImpl},
};

#[derive(thiserror::Error, Debug)]
pub enum SongServiceError {
    #[error(
        "This song has been requested too recently. You can request another song in {0} seconds."
    )]
    SongCooldown(i64),

    #[error("You have recently requested a song. You can request another song in {0} seconds.")]
    UserCooldown(i64),

    #[error("This song is not available")]
    SongAlreadyPlaying(i64),

    #[error("Song not found")]
    SongNotFound(String),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    User(#[from] UserServiceError),
    #[error(transparent)]
    Cooldown(#[from] CooldownServiceError),
    #[error(transparent)]
    Liquidsoap(#[from] LiquidsoapError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

impl ToPublicError for SongServiceError {
    fn as_public(&self) -> Option<crate::dtos::error::PublicError> {
        match self {
            SongServiceError::SongCooldown(seconds) => Some(
                PublicError::with_owned("song-cooldown", self.to_string(), StatusCode::CONFLICT)
                    .with_header(RETRY_AFTER, seconds.to_string()),
            ),
            SongServiceError::UserCooldown(seconds) => Some(
                PublicError::with_owned(
                    "user-cooldown",
                    self.to_string(),
                    StatusCode::TOO_MANY_REQUESTS,
                )
                .with_header(RETRY_AFTER, seconds.to_string()),
            ),
            SongServiceError::SongAlreadyPlaying(seconds) => Some(
                PublicError::with_owned(
                    "song-already-playing",
                    self.to_string(),
                    StatusCode::CONFLICT,
                )
                .with_header(RETRY_AFTER, seconds.to_string()),
            ),
            SongServiceError::SongNotFound(file_hash) => Some(
                PublicError::with_owned("song-not-found", self.to_string(), StatusCode::NOT_FOUND)
                    .with_header("file_hash", file_hash),
            ),
            SongServiceError::User(e) => e.as_public(),
            SongServiceError::Cooldown(e) => e.as_public(),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
pub trait SongService: Send + Sync + 'static {
    async fn request_song(
        &self,
        user_id: UserId,
        file_hash: &str,
    ) -> Result<SongWithCooldownInfo, SongServiceError>;
    async fn get_request_queue(&self) -> Result<Vec<SongDto>, SongServiceError>;
    async fn get_song_history(
        &self,
        pagination: &PaginationParams,
    ) -> Result<Page<SongDto>, SongServiceError>;
}

pub struct SongServiceImpl {
    // repositories
    song_repo: Box<dyn SongRepository>,
    user_repo: Box<dyn UserRepository>,
    song_request_repo: Box<dyn SongRequestRepository>,
    song_history_repo: Box<dyn SongHistoryRepository>,
    tags_repo: Box<dyn TagRepository>,

    // services
    user_service: Arc<dyn UserService>,
    cooldown_service: Arc<dyn CooldownService>,

    liquidsoap_client: Arc<Mutex<LiquidsoapClient>>,
}

impl SongServiceImpl {
    pub fn new(
        song_repo: Box<dyn SongRepository>,
        user_repo: Box<dyn UserRepository>,
        song_request_repo: Box<dyn SongRequestRepository>,
        song_history_repo: Box<dyn SongHistoryRepository>,
        tags_repo: Box<dyn TagRepository>,
        user_service: Arc<dyn UserService>,
        cooldown_service: Arc<dyn CooldownService>,
        liquidsoap_client: Arc<Mutex<LiquidsoapClient>>,
    ) -> Self {
        Self {
            song_repo,
            user_repo,
            song_request_repo,
            song_history_repo,
            tags_repo,
            user_service,
            cooldown_service,
            liquidsoap_client,
        }
    }
}

#[async_trait::async_trait]
impl SongService for SongServiceImpl {
    async fn request_song(
        &self,
        user_id: UserId,
        file_hash: &str,
    ) -> Result<SongWithCooldownInfo, SongServiceError> {
        // ensure user
        self.user_service.get_user(user_id).await?;
        self.user_service.update_user_activity(user_id).await?;

        // check if user has requested a song recently
        let cooldown = SongRequestCooldown;
        if cooldown
            .on_cooldown(&self.cooldown_service, user_id)
            .await?
        {
            // user has requested a song recently
            let expires_at = cooldown
                .get(&self.cooldown_service, user_id)
                .await?
                .expect("Cooldown should be set");
            let now = chrono::Utc::now().naive_utc();
            return Err(SongServiceError::UserCooldown(
                (expires_at - now).num_seconds(),
            ));
        }

        // check if the song requested actually exists
        let Some(song) = self.song_repo.find_by_hash(file_hash).await? else {
            return Err(SongServiceError::SongNotFound(file_hash.to_string()));
        };

        // check if the song requested is already playing
        let currently_playing = self.song_history_repo.recent_plays(1).await?;
        if let Some(playing) = currently_playing.iter().find(|p| p.song_id == file_hash) {
            let now = chrono::Utc::now().naive_utc();
            let elapsed = now - playing.played_at;
            let left = song.duration.round() as i64 - elapsed.num_seconds();
            return Err(SongServiceError::SongAlreadyPlaying(left));
        }

        // check if the song has been requested recently
        let song_cooldown = SongCooldown::new(file_hash, song.duration);
        if song_cooldown.on_cooldown(&self.cooldown_service).await? {
            let expires_at = song_cooldown
                .get(&self.cooldown_service)
                .await?
                .expect("Cooldown should be set");
            let now = chrono::Utc::now().naive_utc();
            return Err(SongServiceError::SongCooldown(
                (expires_at - now).num_seconds(),
            ));
        }

        {
            let mut guard = self.liquidsoap_client.lock().await;
            guard
                .command_with_reconnect(&format!("srq.push {}", &song.file_path))
                .await?;
        }

        self.song_request_repo
            .request_song(file_hash, user_id.into())
            .await?;

        let now = chrono::Utc::now().naive_utc();
        let cooldown_info = CooldownInfo {
            user_cooldown_expires_at: now + cooldown.duration(),
            song_cooldown_expires_at: now + song_cooldown.duration(),
        };
        let song = SongWithCooldownInfo {
            song: song.into(),
            cooldown_info,
        };

        Ok(song)
    }

    async fn get_request_queue(&self) -> Result<Vec<SongDto>, SongServiceError> {
        let mut client = self.liquidsoap_client.lock().await;
        let response = client.command("song_request_queue").await?;

        let queue: Vec<QueueItem> = serde_json::from_str(&response)?;
        let mut songs = Vec::new();

        for item in queue {
            let song = self.song_repo.find_by_path(&item.filename).await?;
            let Some(song) = song else {
                continue;
            };
            songs.push(song.into());
        }

        Ok(songs)
    }

    async fn get_song_history(
        &self,
        pagination: &PaginationParams,
    ) -> Result<Page<SongDto>, SongServiceError> {
        self.song_repo
            .find_recently_played(pagination)
            .await
            .map_err(Into::into)
            .map(|page| page.map(|song| song.into()))
    }
}
