use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fred::pool::RedisPool;
use fred::prelude::PubsubInterface;
use fred::types::{PerformanceConfig, ReconnectPolicy, RedisConfig};

use judeharley::sea_orm::DatabaseConnection;
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

    let db_song = judeharley::Songs::get(&song.filename, &app_state.db)
        .await
        .expect("Failed to query database")
        .expect("Song not found");

    judeharley::PlayedSongs::insert(&db_song, &app_state.db)
        .await
        .expect("Failed to insert played song");

    let _ = app_state
        .redis_pool
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
    redis_pool: RedisPool,
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let config = RedisConfig::from_url(&redis_url).expect("Failed to parse redis url");
    let perf = PerformanceConfig::default();
    let policy = ReconnectPolicy::new_exponential(0, 100, 30_000, 2);
    let redis_pool =
        RedisPool::new(config, Some(perf), Some(policy), 1).expect("Failed to create redis pool");
    redis_pool.connect();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = judeharley::connect_database(&db_url)
        .await
        .expect("Failed to connect to database");

    let app_state = AppState { redis_pool, db };

    let app = axum::Router::new()
        .route("/played", axum::routing::post(played))
        .with_state(app_state);

    info!("Listening on 0.0.0.0:8000");
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
