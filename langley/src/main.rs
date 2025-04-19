use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fred::clients::Pool;
use fred::prelude::{ClientLike, PubsubInterface};

use judeharley::sea_orm::DatabaseConnection;
use judeharley::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Deserialize, Debug)]
struct Song {
    filename: String,
    title: String,
    artist: String,
    album: String,
}

#[derive(Serialize, Debug)]
struct SongResponse {
    success: bool,
}

async fn played(
    State(app_state): State<AppState>,
    Json(song): Json<Song>,
) -> (StatusCode, Json<SongResponse>) {
    if song.filename.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(SongResponse { success: false }),
        );
    }

    let db_song = Songs::get(&song.filename, &app_state.db)
        .await
        .expect("Failed to query database")
        .expect("Song not found");

    PlayedSongs::insert(&db_song, &app_state.db)
        .await
        .expect("Failed to insert played song");

    let client = app_state.redis_pool.next();
    let _ = client
        .publish::<i32, _, _>(
            "byers:status",
            format!("{} - {} - {}", song.album, song.artist, song.title),
        )
        .await;

    debug!("Played song: {}", song.filename);

    (StatusCode::OK, Json(SongResponse { success: true }))
}

#[derive(Clone)]
struct AppState {
    redis_pool: Pool,
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let client = judeharley::redis_pool(&redis_url).expect("Failed to create redis pool");
    let handle = client.init().await.expect("Failed to initialize redis");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = judeharley::connect_database(&db_url)
        .await
        .expect("Failed to connect to database");

    let app_state = AppState { redis_pool: client.clone(), db };

    let app = axum::Router::new()
        .route("/played", axum::routing::post(played))
        .with_state(app_state);

    info!("Listening on 0.0.0.0:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.expect("Failed to bind to 0.0.0.0:8000");
    axum::serve(listener, app.into_make_service()).await.unwrap();

    client.quit().await.expect("Failed to quit Redis");
    handle.await
        .expect("Failed to await join handle")
        .expect("Failed to await Redis quit");
}
