use std::sync::Arc;

use caliborn::{LiquidsoapClient, LiquidsoapError, SeaOrmRepositoryFactory};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::Mutex;

use crate::config::Config;

mod config;

#[derive(thiserror::Error, Debug)]
enum ApplicationError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid configuration: {0}")]
    Figment(#[from] figment::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Hmac(#[from] hmac::digest::InvalidLength),
    #[error(transparent)]
    SeaOrm(#[from] sea_orm::DbErr),
    #[error(transparent)]
    Liquidsoap(#[from] LiquidsoapError),
}

async fn async_main(config: Config) -> Result<(), ApplicationError> {
    let oauth_client = caliborn::build_oauth2_client(
        &config.discord.client_id,
        &config.discord.client_secret,
        &config.discord.redirect_uri,
    )
    .inspect_err(|e| tracing::error!(error = ?e))?;
    let secret: Hmac<Sha256> = Hmac::new_from_slice(config.jwt.secret.as_bytes())
        .inspect_err(|e| tracing::error!(error = ?e))?;
    let hmac_secret: Hmac<Sha256> = Hmac::new_from_slice(config.bot_auth.secret_key.as_bytes())
        .inspect_err(|e| tracing::error!(error = ?e))?;
    let db = sea_orm::Database::connect(&config.database_url)
        .await
        .inspect_err(|e| tracing::error!(error = ?e))?;
    let liquidsoap_client = LiquidsoapClient::new(&config.liquidsoap_socket)
        .await
        .inspect_err(|e| tracing::error!(error = ?e))?;

    let app = caliborn::make_app(
        secret,
        hmac_secret,
        oauth_client,
        Arc::new(SeaOrmRepositoryFactory::new(&db)),
        Arc::new(Mutex::new(liquidsoap_client)),
    );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .inspect_err(|e| tracing::error!(error = ?e))?;

    axum::serve(listener, app)
        .await
        .inspect_err(|e| tracing::error!(error = ?e))?;

    Ok(())
}

fn main() -> Result<(), ApplicationError> {
    let config = Config::new()?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main(config))
}
