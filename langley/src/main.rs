use rocket::serde::{Deserialize, json::Json, Serialize};

use rocket_db_pools::{Database, Connection};
use rocket_db_pools::sqlx;

#[macro_use]
extern crate rocket;

#[derive(Database)]
#[database("byersdb")]
struct ByersDb(sqlx::PgPool);

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Song<'r> {
    filename: &'r str,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct SongResponse {
    success: bool,
}

#[post("/played", data = "<song>")]
async fn played(mut db: Connection<ByersDb>, song: Json<Song<'_>>) -> Json<SongResponse> {
    sqlx::query!("INSERT INTO played_songs (song_id) VALUES ($1)", song.filename)
        .execute(&mut *db)
        .await
        .expect("Failed to query database");
    println!("Played song: {}", song.filename);
    
    Json(SongResponse {
        success: true,
    })
}


#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(ByersDb::init())
        .mount("/", routes![played])
}