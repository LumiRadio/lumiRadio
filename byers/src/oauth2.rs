use async_fred_session::RedisSessionStore;
use axum::{
    extract::{FromRef, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use axum_sessions::{extractors::WritableSession, SessionLayer};
use fred::pool::RedisPool;
use judeharley::{
    discord::{DiscordConnection, MinimalDiscordUser},
    sea_orm::DatabaseConnection,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::Receiver;
use tracing::error;
use tracing_unwrap::ResultExt;

use crate::{app_config::DiscordConfig, prelude::Error};

static OAUTH2_SUCCESS_HTML: &str = include_str!("static/oauth2_success.html");
static OAUTH2_FAILED_CSRF_HTML: &str = include_str!("static/oauth2_csrf.html");
static OAUTH2_FAILED_DISCORD_HTML: &str = include_str!("static/oauth2_discord.html");

#[derive(FromRef, Debug, Clone)]
struct AppState {
    db: DatabaseConnection,
    discord_config: DiscordConfig,
}

#[derive(serde::Deserialize)]
struct OAuth2CallbackParams {
    code: String,
    state: String,
}

fn oauth2_client(client_id: &str, client_secret: &str) -> BasicClient {
    BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
            .expect_or_log("Failed to parse auth url"),
        Some(
            TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
                .expect_or_log("Failed to parse token url"),
        ),
    )
}

#[derive(Deserialize, Debug)]
struct OAuth2LoginParams {
    next: Option<String>,
}

async fn oauth2_login(
    State(discord): State<DiscordConfig>,
    mut session: WritableSession,
    Query(login_params): Query<OAuth2LoginParams>,
) -> Redirect {
    let client = oauth2_client(&discord.client_id, &discord.client_secret);

    let (auth_url, csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(vec![
            Scope::new("identify".to_string()),
            Scope::new("connections".to_string()),
        ])
        .url();
    session
        .insert("state", csrf.secret())
        .expect_or_log("Failed to insert csrf token");

    if let Some(next) = login_params.next {
        session
            .insert("next", next)
            .expect_or_log("Failed to insert next");
    }

    Redirect::to(auth_url.as_ref())
}

async fn oauth2_callback(
    State(db): State<DatabaseConnection>,
    State(discord): State<DiscordConfig>,
    Query(params): Query<OAuth2CallbackParams>,
    mut session: WritableSession,
) -> impl IntoResponse {
    let Some(csrf) = session.get::<String>("state") else {
        return (
            StatusCode::BAD_REQUEST,
            Html(OAUTH2_FAILED_CSRF_HTML.to_string()),
        )
            .into_response();
    };

    if csrf != params.state {
        return (
            StatusCode::BAD_REQUEST,
            Html(OAUTH2_FAILED_CSRF_HTML.to_string()),
        )
            .into_response();
    }
    let client = oauth2_client(&discord.client_id, &discord.client_secret);
    let Ok(token) = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await
    else {
        return (
            StatusCode::BAD_REQUEST,
            Html(OAUTH2_FAILED_DISCORD_HTML.to_string()),
        )
            .into_response();
    };
    session.remove("state");
    session
        .insert("token", token.clone())
        .expect_or_log("Failed to insert token");

    let connections = DiscordConnection::fetch(token.access_token().secret())
        .await
        .expect_or_log("Failed to fetch Discord connections");
    let youtube_connections = connections
        .into_iter()
        .filter(|c| c.kind == "youtube")
        .collect::<Vec<_>>();

    let logged_in_user = MinimalDiscordUser::fetch(token.access_token().secret())
        .await
        .expect_or_log("Failed to fetch Discord user");
    let user = judeharley::Users::get_or_insert(logged_in_user.id.parse().unwrap_or_log(), &db)
        .await
        .expect_or_log("Failed to get or insert user");
    user.insert_channels(&youtube_connections, &db)
        .await
        .expect_or_log("Failed to insert channels");

    let next = session.get::<String>("next");
    if let Some(next) = next {
        session.remove("next");
        return Redirect::to(&next).into_response();
    }

    (StatusCode::OK, Html(OAUTH2_SUCCESS_HTML.to_string())).into_response()
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
enum ApiResponse<T> {
    Success { data: T },
    Error { error: String },
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Serialize, Debug)]
struct Song {
    id: String,
    title: String,
    artist: String,
    album: String,
    duration: f64,
}

impl From<judeharley::Songs> for Song {
    fn from(value: judeharley::Songs) -> Self {
        Self {
            id: value.file_hash,
            title: value.title,
            artist: value.artist,
            album: value.album,
            duration: value.duration,
        }
    }
}

async fn song_list(State(app_state): State<AppState>) -> ApiResponse<Vec<Song>> {
    let songs = match judeharley::Songs::get_all(&app_state.db).await {
        Ok(songs) => songs,
        Err(e) => {
            error!("Failed to fetch songs: {}", e);
            return ApiResponse::Error {
                error: "Failed to fetch songs".to_string(),
            };
        }
    };

    ApiResponse::Success {
        data: songs.into_iter().map(Into::into).collect(),
    }
}

pub async fn oauth2_server(
    secret: String,
    db: DatabaseConnection,
    redis: RedisPool,
    discord_config: DiscordConfig,
    ctrl_c: Receiver<()>,
) -> Result<(), Error> {
    let cookie_store = RedisSessionStore::from_pool(redis, Some("byers-session/".into()));
    let session_layer = SessionLayer::new(cookie_store, secret.as_bytes())
        .with_same_site_policy(axum_sessions::SameSite::Lax);

    let app = Router::new()
        .route("/oauth2/callback", get(oauth2_callback))
        .route("/oauth2/login", get(oauth2_login))
        .route("/api/songs", get(song_list))
        .with_state(AppState { db, discord_config })
        .layer(session_layer);

    axum::Server::bind(&"0.0.0.0:8000".parse()?)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            ctrl_c.await.ok();
        })
        .await?;

    Ok(())
}
