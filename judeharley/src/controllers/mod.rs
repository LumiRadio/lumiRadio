use sea_orm::FromQueryResult;

pub mod cans;
pub mod connected_youtube_accounts;
pub mod favourite_songs;
pub mod played_songs;
pub mod server_channel_config;
pub mod server_config;
pub mod server_role_config;
pub mod slcb_currency;
pub mod slcb_rank;
pub mod song_requests;
pub mod song_tags;
pub mod songs;
pub mod users;

#[derive(FromQueryResult)]
pub struct CountQuery {
    pub count: i64,
}
