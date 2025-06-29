use axum::response::IntoResponse;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{dtos::json, entities};

#[derive(Serialize, ToSchema)]
pub struct SongDto {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    pub bitrate: i32,
}

impl From<entities::songs::Model> for SongDto {
    fn from(value: entities::songs::Model) -> Self {
        Self {
            id: value.file_hash,
            title: value.title,
            artist: value.artist,
            album: value.album,
            duration: value.duration,
            bitrate: value.bitrate,
        }
    }
}

#[derive(Serialize)]
pub struct SongListDto(Vec<SongDto>);

impl From<Vec<SongDto>> for SongListDto {
    fn from(value: Vec<SongDto>) -> Self {
        Self(value)
    }
}

impl IntoResponse for SongListDto {
    fn into_response(self) -> axum::response::Response {
        json(self).into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct PlayInfo {
    pub play_count: i32,
    pub request_count: i32,
    pub last_played_at: NaiveDateTime,
    pub last_requested_at: NaiveDateTime,
    pub on_cooldown: bool,
    pub cooldown_expires_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct SongWithPlayInfo {
    #[serde(flatten)]
    pub song: SongDto,
    #[serde(flatten)]
    pub play_info: PlayInfo,
}

impl SongWithPlayInfo {
    pub fn new(song: SongDto, play_info: PlayInfo) -> Self {
        Self { song, play_info }
    }
}

#[derive(Serialize, ToSchema)]
pub struct CooldownInfo {
    pub user_cooldown_expires_at: NaiveDateTime,
    pub song_cooldown_expires_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct SongWithCooldownInfo {
    #[serde(flatten)]
    pub song: SongDto,
    #[serde(flatten)]
    pub cooldown_info: CooldownInfo,
}

impl SongWithCooldownInfo {
    pub fn new(song: SongDto, cooldown_info: CooldownInfo) -> Self {
        Self {
            song,
            cooldown_info,
        }
    }
}

impl IntoResponse for SongWithCooldownInfo {
    fn into_response(self) -> axum::response::Response {
        json(self).into_response()
    }
}

#[derive(Deserialize, ToSchema)]
pub struct SongRequest {
    pub file_hash: String,
}
