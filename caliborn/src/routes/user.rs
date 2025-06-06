use axum::{Router, extract::State, response::Response, routing::get};

use crate::{
    AppState, ServiceRegistry,
    dtos::{
        error::{CalibornResult, ErrorResponse},
        json,
        users::UserDto,
    },
    services::auth::{AuthenticatedUser, authenticate},
};

#[utoipa::path(
    get,
    path = "/user/me",
    responses(
        (status = 200, description = "User was successfully retrieved", body = UserDto),
        (status = 401, description = "An authorization error occurred (e.g. invalid access token)", body = ErrorResponse, example = json!({"message": "Invalid access token", "error": "Unauthorized"})),
        (status = 500, description = "An internal server error occurred", body = ErrorResponse, example = json!({"message": "Internal server error", "error": "Internal Server Error"}))
    )
)]
#[axum::debug_handler]
pub async fn me(
    AuthenticatedUser(actor): AuthenticatedUser,
    State(state): State<AppState>,
) -> CalibornResult<UserDto> {
    let user_id = actor.user_id();

    let user_service = state.service_registry.user_service();
    let user = user_service.get_user(user_id).await?;

    Ok(user)
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(me))
        .layer(axum::middleware::from_fn_with_state(state, authenticate))
}
