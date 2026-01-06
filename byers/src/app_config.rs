use serde::Deserialize;

fn default_environment() -> String {
    "development".into()
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub discord_token: String,
    pub database_url: String,
    pub redis_url: String,

    pub discord: DiscordConfig,
    pub secret: String,

    pub sentry_dsn: Option<String>,
    #[serde(default = "default_environment")]
    pub environment: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DiscordConfig {
    pub client_id: String,
    pub client_secret: String,

    pub emoji: EmojiConfig,
    pub admin_user_ids: Vec<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmojiConfig {
    pub d6_1: String,
    pub d6_2: String,
    pub d6_3: String,
    pub d6_4: String,
    pub d6_5: String,
    pub d6_6: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let config = config::Config::builder()
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .list_separator(",")
                    .with_list_parse_key("discord.admin_user_ids")
                    .try_parsing(true),
            )
            .build()
            .unwrap();

        config.try_deserialize().unwrap()
    }
}
