use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub discord_token: String,
    pub database_url: String,

    pub liquidsoap: LiquidsoapConfig,

    pub google: GoogleConfig
}

#[derive(Deserialize, Debug)]
pub struct LiquidsoapConfig {
    pub host: String,
    pub port: u16
}

#[derive(Deserialize, Debug)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String
}

impl AppConfig {
    pub fn from_env() -> Self {
        let config = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()
            .unwrap();

        config.try_deserialize().unwrap()
    }
}