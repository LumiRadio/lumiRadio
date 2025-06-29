use axum::{
    Router,
    extract::State,
    routing::{get, post},
};

use crate::{
    AppState, ServiceRegistry,
    dtos::{
        Query,
        error::CalibornResult,
        page::{Page, PaginationParams},
        songs::{SongDto, SongListDto, SongRequest, SongWithCooldownInfo},
    },
    services::{
        auth::{AuthenticatedUser, authenticate},
        songs::SongService,
        users::UserService,
    },
};

#[axum::debug_handler]
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

#[axum::debug_handler]
pub async fn get_request_queue(
    State(registry): State<ServiceRegistry>,
) -> CalibornResult<SongListDto> {
    let song_service = registry.song_service();
    song_service
        .get_request_queue()
        .await
        .map(SongListDto::from)
        .map_err(Into::into)
}

#[axum::debug_handler]
pub async fn get_song_history(
    State(registry): State<ServiceRegistry>,
    Query(pagination): Query<PaginationParams>,
) -> CalibornResult<Page<SongDto>> {
    let song_service = registry.song_service();
    song_service
        .get_song_history(&pagination)
        .await
        .map_err(Into::into)
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/request", post(request_song))
        .layer(axum::middleware::from_fn_with_state(state, authenticate))
        .route("/queue", get(get_request_queue))
        .route("/history", get(get_song_history))
}
