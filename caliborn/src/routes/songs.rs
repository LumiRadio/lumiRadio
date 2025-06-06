use axum::{Router, extract::State};

use crate::{
    AppState, ServiceRegistry,
    dtos::{
        Query,
        error::CalibornResult,
        songs::{SongRequest, SongWithCooldownInfo},
    },
    services::auth::AuthenticatedUser,
};

pub async fn request_song(
    AuthenticatedUser(actor): AuthenticatedUser,
    State(registry): State<ServiceRegistry>,
    Query(song_request): Query<SongRequest>,
) -> CalibornResult<SongWithCooldownInfo> {
    let user_service = registry.user_service();
    let song_service = registry.song_service();

    // ensure user
    user_service.get_user(actor.user_id()).await?;

    let song_with_cooldown = song_service
        .request_song(actor.user_id(), &song_request.file_hash)
        .await?;
    Ok(song_with_cooldown)
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
}
