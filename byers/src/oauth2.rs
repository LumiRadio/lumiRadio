use async_fred_session::RedisSessionStore;
use axum::{
    extract::{FromRef, Query, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
    Router,
};
use axum_sessions::{extractors::WritableSession, SessionLayer};
use fred::pool::RedisPool;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, Scope, TokenUrl, TokenResponse,
};
use sqlx::PgPool;
use tokio::sync::oneshot::Receiver;
use tracing::info;
use tracing_unwrap::ResultExt;

use crate::{app_config::DiscordConfig, prelude::Error, discord::{DiscordConnection, MinimalDiscordUser}, db::DbUser};

static OAUTH2_SUCCESS_HTML: &str = include_str!("static/oauth2_success.html");
static OAUTH2_FAILED_CSRF_HTML: &str = include_str!("static/oauth2_csrf.html");
static OAUTH2_FAILED_DISCORD_HTML: &str = include_str!("static/oauth2_discord.html");

#[derive(FromRef, Debug, Clone)]
struct AppState {
    db: PgPool,
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

async fn oauth2_login(
    State(discord): State<DiscordConfig>,
    mut session: WritableSession,
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

    Redirect::to(auth_url.as_ref())
}

async fn oauth2_callback(
    State(db): State<PgPool>,
    State(discord): State<DiscordConfig>,
    Query(params): Query<OAuth2CallbackParams>,
    mut session: WritableSession,
) -> (StatusCode, Html<String>) {
    let Some(csrf) = session.get::<String>("state") else {
        return (StatusCode::BAD_REQUEST, Html(OAUTH2_FAILED_CSRF_HTML.to_string()));
    };

    if csrf != params.state {
        return (
            StatusCode::BAD_REQUEST,
            Html(OAUTH2_FAILED_CSRF_HTML.to_string()),
        );
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
        );
    };
    session.destroy();

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
    let user = DbUser::fetch_or_insert(&db, logged_in_user.id.parse().unwrap_or_log()).await
        .expect_or_log("Failed to fetch or insert user");
    user.add_linked_channels(&db, youtube_connections).await
        .expect_or_log("Failed to add linked channels");

    (StatusCode::OK, Html(OAUTH2_SUCCESS_HTML.to_string()))
}

pub async fn oauth2_server(
    secret: String,
    db: PgPool,
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
