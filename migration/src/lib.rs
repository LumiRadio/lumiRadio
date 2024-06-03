pub use sea_orm_migration::prelude::*;

mod m20240506_215517_initial;
mod m20240530_174050_edit_users_change_watched_hours;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240506_215517_initial::Migration),
            Box::new(m20240530_174050_edit_users_change_watched_hours::Migration),
        ]
    }
}
