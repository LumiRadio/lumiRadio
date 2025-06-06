pub use sea_orm_migration::prelude::*;

mod m20240506_215517_initial;
mod m20240530_174050_edit_users_change_watched_hours;
mod m20240706_153336_edit_server_channel_config_add_last_message_sent;
mod m20250420_155859_edit_users_add_username;
mod m20250420_222941_create_token_storage;
mod m20250422_120742_create_api_keys;
mod m20250423_082923_edit_users_remove_grist;
mod m20250424_162423_edit_api_keys_add_hash;
mod m20250429_074227_create_cooldowns;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240506_215517_initial::Migration),
            Box::new(m20240530_174050_edit_users_change_watched_hours::Migration),
            Box::new(m20240706_153336_edit_server_channel_config_add_last_message_sent::Migration),
            Box::new(m20250420_155859_edit_users_add_username::Migration),
            Box::new(m20250420_222941_create_token_storage::Migration),
            Box::new(m20250422_120742_create_api_keys::Migration),
            Box::new(m20250423_082923_edit_users_remove_grist::Migration),
            Box::new(m20250424_162423_edit_api_keys_add_hash::Migration),
            Box::new(m20250429_074227_create_cooldowns::Migration),
        ]
    }
}
