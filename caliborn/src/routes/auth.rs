use axum::{Router, extract::State, routing::post};

use crate::{
    AppState,
    dtos::{
        Json,
        auth::{DiscordLoginRequest, UserToken},
        error::{CalibornResult, ErrorResponse},
    },
    services::{ServiceRegistry, auth::AuthService},
};

/// Logs in a user via Discord
///
/// This endpoint is used to exchange the authorization code received from Discord for an access token.
///
/// The authorization code is obtained by redirecting the user to the [Discord authorization URL](https://discord.com/developers/docs/topics/oauth2#authorization-code-flow).
///
/// # Parameters
///
/// * `code` - The authorization code received from Discord
///
/// # Returns
///
/// A `200 OK` response containing the access token, user ID, and expiration time.
///
/// # Errors
///
/// * `401 Unauthorized` - An authorization error occurred (e.g. invalid authorization code)
/// * `500 Internal Server Error` - An internal server error occurred
#[utoipa::path(
    post,
    path = "/auth/discord/login",
    request_body = DiscordLoginRequest,
    responses(
        (status = 200, description = "Discord login was successful", body = UserToken),
        (status = 401, description = "An authorization error occurred (e.g. invalid authorization code)", body = ErrorResponse, example = json!({"message": "Invalid authorization code", "error": "Unauthorized"})),
        (status = 500, description = "An internal server error occurred", body = ErrorResponse, example = json!({"message": "Internal server error", "error": "Internal Server Error"}))
    )
)]
#[axum::debug_handler]
pub async fn discord_login(
    State(registry): State<ServiceRegistry>,
    Json(payload): Json<DiscordLoginRequest>,
) -> CalibornResult<UserToken> {
    let auth_service = registry.auth_service();
    let token = auth_service.login_user(&payload.code).await?;

    Ok(token)
}

/// Builds a router for the authentication routes.
///
/// # Returns
///
/// An `axum::Router` containing the authentication routes.
///
/// # Routes
///
/// * `POST /auth/discord/login` - Logs in a user via Discord
pub fn routes() -> Router<AppState> {
    Router::new().route("/discord/login", post(discord_login))
}
