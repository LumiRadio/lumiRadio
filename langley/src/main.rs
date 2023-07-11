use fred::prelude::{ClientLike, PubsubInterface, RedisClient};
use fred::types::{PerformanceConfig, ReconnectPolicy, RedisConfig};
use rocket::fairing::AdHoc;
use rocket::log::private::info;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;

use rocket_db_pools::sqlx;
use rocket_db_pools::{Connection, Database};

#[macro_use]
extern crate rocket;

#[derive(Database)]
#[database("byersdb")]
struct ByersDb(sqlx::PgPool);

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Song<'r> {
    filename: &'r str,
    title: &'r str,
    artist: &'r str,
    album: &'r str,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct SongResponse {
    success: bool,
}

#[post("/played", data = "<song>")]
async fn played(
    mut db: Connection<ByersDb>,
    redis_state: &State<RedisClient>,
    song: Json<Song<'_>>,
) -> Json<SongResponse> {
    if song.filename.is_empty() {
        return Json(SongResponse { success: false });
    }

    let _ = redis_state
        .publish::<i32, _, _>("byers:status", format!("{} - {}", song.album, song.title))
        .await;

    sqlx::query!(
        "INSERT INTO played_songs (song_id) VALUES ($1)",
        song.filename
    )
    .execute(&mut *db)
    .await
    .expect("Failed to query database");
    println!("Played song: {}", song.filename);

    Json(SongResponse { success: true })
}

#[launch]
async fn rocket() -> _ {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let config = RedisConfig::from_url(&redis_url).expect("Failed to parse redis url");
    let perf = PerformanceConfig::default();
    let policy = ReconnectPolicy::new_exponential(0, 100, 30_000, 2);
    let redis_client = RedisClient::new(config, Some(perf), Some(policy));

    let connect_handle = redis_client.connect();
    redis_client
        .wait_for_connect()
        .await
        .expect("Failed to connect to redis");

    rocket::build()
        .manage(redis_client.clone())
        .attach(ByersDb::init())
        .attach(AdHoc::on_shutdown("shutdown redis", |_| {
            Box::pin(async move {
                redis_client.quit().await.expect("Failed to quit redis");
                connect_handle
                    .await
                    .expect("Failed to wait for redis to quit")
                    .expect("Failed to quit redis");
            })
        }))
        .mount("/", routes![played])
}
