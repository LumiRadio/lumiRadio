use std::usize;

use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, FromRequestParts, Request, State},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use oauth2::{AuthorizationCode, TokenResponse};
use prefixed_api_key::{PakControllerOsSha256, PrefixedApiKey, PrefixedApiKeyController};
use rand::Rng;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::{
    AppState, DiscordOAuthClient,
    dtos::{
        Json,
        auth::{ApiKeyDto, UserToken},
        error::{ApiError, PublicError, ToPublicError},
    },
    repositories::{RepositoryError, users::UserRepository},
};

use super::UserId;

const MAX_SKEW_MS: chrono::Duration = chrono::Duration::minutes(5);

#[derive(thiserror::Error, Debug)]
pub enum AuthServiceError {
    #[error("Unexpected Discord API error")]
    DiscordApiError(#[from] serenity::Error),
    #[error("Misconfigured OAuth client")]
    MisconfiguredOAuthClient,
    #[error("Invalid authorization code")]
    InvalidAuthorizationCode,
    #[error("Invalid OAuth request")]
    InvalidOAuthRequest,
    #[error("Invalid scope specified")]
    InvalidScope,
    #[error("Invalid expiration time")]
    InvalidExpirationTime,
    #[error("Missing refresh token")]
    MissingRefreshToken,
    #[error("Other OAuth-related error: {0}")]
    OtherOAuthError(String),
    #[error("Invalid JWT token")]
    InvalidJwtToken,
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Invalid HMAC timestamp")]
    InvalidHmacTimestamp,
    #[error("Invalid HMAC signature")]
    InvalidHmacSignature,
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl From<&oauth2::basic::BasicErrorResponseType> for AuthServiceError {
    fn from(value: &oauth2::basic::BasicErrorResponseType) -> Self {
        match value {
            oauth2::basic::BasicErrorResponseType::InvalidClient => {
                AuthServiceError::MisconfiguredOAuthClient
            }
            oauth2::basic::BasicErrorResponseType::InvalidGrant => {
                AuthServiceError::InvalidAuthorizationCode
            }
            oauth2::basic::BasicErrorResponseType::InvalidRequest => {
                AuthServiceError::InvalidOAuthRequest
            }
            oauth2::basic::BasicErrorResponseType::InvalidScope => AuthServiceError::InvalidScope,
            oauth2::basic::BasicErrorResponseType::UnauthorizedClient => {
                AuthServiceError::InvalidOAuthRequest
            }
            oauth2::basic::BasicErrorResponseType::UnsupportedGrantType => {
                AuthServiceError::InvalidOAuthRequest
            }
            oauth2::basic::BasicErrorResponseType::Extension(e) => {
                AuthServiceError::OtherOAuthError(e.to_string())
            }
        }
    }
}

impl ToPublicError for AuthServiceError {
    fn as_public(&self) -> Option<PublicError> {
        match self {
            AuthServiceError::InvalidAuthorizationCode => Some(PublicError::new(
                "invalid_authorization_code",
                "Invalid authorization code",
                StatusCode::FORBIDDEN,
            )),
            AuthServiceError::InvalidJwtToken => Some(PublicError::new(
                "invalid_jwt_token",
                "Invalid JWT token",
                StatusCode::UNAUTHORIZED,
            )),
            AuthServiceError::InvalidApiKey => Some(PublicError::new(
                "invalid_api_key",
                "Invalid API key",
                StatusCode::UNAUTHORIZED,
            )),
            AuthServiceError::InvalidHmacSignature => Some(PublicError::new(
                "invalid_hmac_signature",
                "Invalid HMAC signature",
                StatusCode::UNAUTHORIZED,
            )),
            AuthServiceError::InvalidHmacTimestamp => Some(PublicError::new(
                "invalid_hmac_timestamp",
                "Invalid HMAC timestamp",
                StatusCode::UNAUTHORIZED,
            )),
            _ => None,
        }
    }
}

pub struct AuthService {
    user_repo: Box<dyn UserRepository>,
    oauth_client: DiscordOAuthClient,
    jwt_secret: Hmac<Sha256>,
    hmac_secret: Hmac<Sha256>,
    http_client: reqwest::Client,
    key_generator: PakControllerOsSha256,
}

impl AuthService {
    pub fn new(
        user_repo: Box<dyn UserRepository>,
        oauth_client: DiscordOAuthClient,
        jwt_secret: Hmac<Sha256>,
        hmac_secret: Hmac<Sha256>,
    ) -> Self {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        let key_generator = PrefixedApiKeyController::configure()
            .prefix("ak".to_string())
            .seam_defaults()
            .finalize()
            .unwrap();

        Self {
            user_repo,
            oauth_client,
            jwt_secret,
            hmac_secret,
            http_client,
            key_generator,
        }
    }

    pub async fn login_user(&self, code: &str) -> Result<UserToken, AuthServiceError> {
        let token_response = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&self.http_client)
            .await
            .map_err(|error| match error {
                oauth2::RequestTokenError::ServerResponse(response_error) => {
                    response_error.error().into()
                }
                error => AuthServiceError::OtherOAuthError(error.to_string()),
            })?;

        let expiration = chrono::Duration::from_std(
            token_response
                .expires_in()
                .unwrap_or(std::time::Duration::from_secs(0)),
        )
        .map_err(|_| AuthServiceError::InvalidExpirationTime)?;
        let token = token_response.access_token();
        let refresh_token = token_response
            .refresh_token()
            .ok_or(AuthServiceError::MissingRefreshToken)?;
        let expires_at = Utc::now() + expiration;

        let discord_client =
            serenity::http::HttpBuilder::new(format!("Bearer {}", token.secret())).build();
        let current_user = discord_client.get_current_user().await?;
        let user_id = current_user.id.get();

        // let storage = if self
        //     .token_storage_repo
        //     .get_by_user_id(user_id as i64)
        //     .await?
        //     .is_some()
        // {
        //     self.token_storage_repo
        //         .update(
        //             user_id as i64,
        //             token.secret(),
        //             refresh_token.secret(),
        //             expires_at.naive_utc(),
        //         )
        //         .await?
        // } else {
        //     self.token_storage_repo
        //         .insert(
        //             user_id as i64,
        //             token.secret(),
        //             refresh_token.secret(),
        //             expires_at.naive_utc(),
        //         )
        //         .await?
        // };

        let claims = Claims::new(user_id.into(), expiration);
        let token = claims
            .sign(&self.jwt_secret)
            .map_err(|_| AuthServiceError::InvalidJwtToken)?;

        Ok(UserToken {
            token,
            user_id,
            expires_in: expiration.num_seconds() as u64,
            expires_at: expires_at.timestamp() as u64,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthServiceError> {
        Claims::verify(token, &self.jwt_secret).map_err(|_| AuthServiceError::InvalidJwtToken)
    }

    pub async fn create_api_key(
        &self,
        user_id: UserId,
        description: &str,
    ) -> Result<ApiKeyDto, AuthServiceError> {
        let (pak, hash) = self.key_generator.generate_key_and_hash();

        let key = self
            .user_repo
            .create_api_key(user_id.into(), pak.short_token(), &hash, description)
            .await?;

        Ok(ApiKeyDto {
            api_key: pak.to_string(),
            description: key.description,
        })
    }

    pub async fn check_api_key(&self, api_key: &str) -> Result<i64, AuthServiceError> {
        let pak: PrefixedApiKey = api_key
            .try_into()
            .map_err(|_| AuthServiceError::InvalidApiKey)?;
        let user = self.user_repo.find_by_api_key(pak.short_token()).await?;

        let Some((key, user)) = user else {
            return Err(AuthServiceError::InvalidApiKey);
        };

        if self.key_generator.check_hash(&pak, &key.hash) {
            Ok(user.id)
        } else {
            Err(AuthServiceError::InvalidApiKey)
        }
    }

    pub fn verify_hmac(
        &self,
        body: &[u8],
        signature: &[u8],
        timestamp: &str,
        method: &str,
        path: &str,
    ) -> Result<(), AuthServiceError> {
        let sent_at = match DateTime::parse_from_rfc3339(timestamp) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return Err(AuthServiceError::InvalidHmacTimestamp),
        };

        let now = Utc::now();
        if (now - sent_at).num_seconds().abs() > MAX_SKEW_MS.num_seconds() {
            return Err(AuthServiceError::InvalidHmacTimestamp);
        }

        let body_hash = hex::encode(Sha256::digest(&body));
        let canonical = format!("{}\n{}\n{}\n{}", method, path, body_hash, timestamp,);

        let mut mac = self.hmac_secret.clone();
        mac.update(canonical.as_bytes());
        let expected = mac.finalize().into_bytes();

        let supplied = match hex::decode(signature) {
            Ok(bytes) => bytes,
            Err(_) => return Err(AuthServiceError::InvalidHmacSignature),
        };
        if supplied.len() != expected.len() || supplied.ct_eq(&expected).unwrap_u8() == 0 {
            return Err(AuthServiceError::InvalidHmacSignature);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    #[serde(flatten)]
    standard: jwt::RegisteredClaims,
}

impl Claims {
    pub fn new(user_id: UserId, expiry: chrono::Duration) -> Self {
        let now = chrono::Utc::now();
        let expiry = now + expiry;

        Self {
            standard: jwt::RegisteredClaims {
                issuer: Some("caliborn".to_string()),
                subject: Some(user_id.to_string()),
                audience: Some("calliope".to_string()),
                expiration: Some(expiry.timestamp() as u64),
                not_before: Some(now.timestamp() as u64),
                issued_at: Some(now.timestamp() as u64),
                json_web_token_id: None,
            },
        }
    }

    pub fn sign(&self, secret: &Hmac<Sha256>) -> Result<String, jwt::Error> {
        self.sign_with_key(secret)
    }

    pub fn verify(token: &str, secret: &Hmac<Sha256>) -> Result<Self, jwt::Error> {
        token.verify_with_key(secret)
    }

    pub fn is_valid_user(&self) -> bool {
        self.standard.subject.is_some()
            && self.standard.audience == Some("calliope".to_string())
            && self.valid()
    }

    pub fn valid(&self) -> bool {
        let now = chrono::Utc::now();

        if let Some(nbf) = self.standard.not_before.as_ref() {
            if *nbf > now.timestamp() as u64 {
                return false;
            }
        }

        if let Some(exp) = self.standard.expiration.as_ref() {
            if *exp < now.timestamp() as u64 {
                return false;
            }
        }

        true
    }
}

pub struct AuthenticatedUser(pub Actor);

impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let extension =
            parts
                .extensions
                .get::<Actor>()
                .ok_or(ApiError::Internal(anyhow::anyhow!(
                    "missing actor extension (perhaps middleware is not used)"
                )))?;
        Ok(AuthenticatedUser(extension.clone()))
    }
}

#[derive(Clone)]
pub enum Actor {
    User { user_id: UserId },
    Bot { user_id: UserId },
}

impl Actor {
    pub fn user_id(&self) -> UserId {
        match self {
            Actor::User { user_id } => *user_id,
            Actor::Bot { user_id } => *user_id,
        }
    }
}

pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if let (Some(signature), Some(timestamp), Some(user_id)) = (
        request
            .headers()
            .get("X-Caliborn-Signature")
            .map(|s| s.as_bytes().to_vec()),
        request
            .headers()
            .get("X-Caliborn-Timestamp")
            .map(|s| s.to_str().unwrap_or_default().to_string()),
        request
            .headers()
            .get("X-Caliborn-User-Id")
            .map(|s| s.to_str().unwrap_or_default().to_string()),
    ) {
        let user_id = user_id.parse::<u64>().map_err(|_| {
            ApiError::Public(PublicError::new(
                "missing_user_id_header",
                "The X-Caliborn-User-Id header is not set",
                StatusCode::UNAUTHORIZED,
            ))
        })?;

        let (parts, body) = request.into_parts();
        let method = parts.method.as_str();
        let bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("reading body failed")))?;
        state.service_registry.auth_service().verify_hmac(
            &bytes,
            &signature,
            &timestamp,
            method,
            parts.uri.path(),
        )?;

        let mut req = Request::from_parts(parts, Body::from(bytes));
        req.extensions_mut().insert(Actor::Bot {
            user_id: user_id.into(),
        });
        return Ok(next.run(req).await);
    }

    if let Some(auth) = request.headers().get(axum::http::header::AUTHORIZATION) {
        let token = auth.to_str().unwrap_or_default().to_string();
        let stripped = token
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Public(PublicError::new(
                "invalid-auth-header",
                "Invalid authentication header",
                StatusCode::UNAUTHORIZED,
            )))?;

        if stripped.starts_with("ak_") {
            let user_id = state
                .service_registry
                .auth_service()
                .check_api_key(stripped)
                .await?;
            request.extensions_mut().insert(Actor::User {
                user_id: user_id.into(),
            });
            return Ok(next.run(request).await);
        } else {
            let claims = state
                .service_registry
                .auth_service()
                .verify_token(&stripped)?;
            if !claims.is_valid_user() {
                return Err(ApiError::Public(PublicError::new(
                    "invalid-auth-header",
                    "Invalid authentication header",
                    StatusCode::UNAUTHORIZED,
                )));
            }

            let user_id = {
                let Some(sub) = claims.standard.subject.as_ref() else {
                    return Err(ApiError::Public(PublicError::new(
                        "invalid-auth-header",
                        "Invalid authentication header",
                        StatusCode::UNAUTHORIZED,
                    )));
                };
                sub.parse::<u64>().map_err(|_| {
                    ApiError::Public(PublicError::new(
                        "invalid-auth-header",
                        "Invalid authentication header",
                        StatusCode::UNAUTHORIZED,
                    ))
                })?
            };
            request.extensions_mut().insert(Actor::User {
                user_id: user_id.into(),
            });
            return Ok(next.run(request).await);
        }
    }

    Err(ApiError::Public(PublicError::new(
        "invalid-auth-header",
        "Invalid authentication header",
        StatusCode::UNAUTHORIZED,
    )))
}
