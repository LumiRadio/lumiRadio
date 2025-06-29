use axum::{
    Router,
    extract::State,
    routing::{get, post},
};

use crate::{
    AppState, ServiceRegistry,
    dtos::{
        cans::CanCountDto,
        error::{CalibornResult, ErrorResponse},
    },
    services::{
        auth::{AuthenticatedUser, authenticate},
        cans::CansService,
    },
};

/// Get the current can count.
///
/// # Returns
///
/// A 200 OK response containing the current can count.
///
/// # Errors
///
/// * `500 Internal Server Error` - An internal server error occurred
#[utoipa::path(
    get,
    path = "/cans/count",
    responses(
        (status = 200, description = "Current can count was successfully retrieved", body = CanCountDto),
        (status = 500, description = "An internal server error occurred", body = ErrorResponse, example = json!({"message": "Internal server error", "error": "Internal Server Error"}))
    )
)]
pub async fn get_can_count(State(registry): State<ServiceRegistry>) -> CalibornResult<CanCountDto> {
    let can_service = registry.can_service();

    let count = can_service.count().await?;
    Ok(CanCountDto { count })
}

/// Add a can to Can Town.
///
/// # Returns
///
/// A 200 OK response containing the new can count.
///
/// # Errors
///
/// * `500 Internal Server Error` - An internal server error occurred
/// * `401 Unauthorized` - An authorization error occurred (e.g. invalid access token)
#[utoipa::path(
    post,
    path = "/cans/add",
    responses(
        (status = 200, description = "Can was successfully added", body = CanCountDto),
        (status = 500, description = "An internal server error occurred", body = ErrorResponse, example = json!({"message": "Internal server error", "error": "Internal Server Error"})),
        (status = 401, description = "An authorization error occurred (e.g. invalid access token)", body = ErrorResponse, example = json!({"message": "Invalid access token", "error": "Unauthorized"}))
    ),
    security(
        ("user_jwt" = []),
        ("user_api_key" = [])
    )
)]
pub async fn add_can(
    AuthenticatedUser(actor): AuthenticatedUser,
    State(registry): State<ServiceRegistry>,
) -> CalibornResult<CanCountDto> {
    let can_service = registry.can_service();

    can_service.add(actor.user_id()).await?;
    let count = can_service.count().await?;
    Ok(CanCountDto { count })
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/count", get(get_can_count))
        .route("/add", post(add_can))
        .layer(axum::middleware::from_fn_with_state(state, authenticate))
}
