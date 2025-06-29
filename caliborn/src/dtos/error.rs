//! Error handling types for the Caliborn API.
//!
//! This module provides error types, result aliases, and response handling for
//! the API. It includes error categorization, serialization, and conversion to
//! HTTP responses.
//!
//! # Examples
//!
//! ```rust
//! # use caliborn::dtos::error::{CalibornError, CalibornResult};
//! fn may_fail() -> CalibornResult<String> {
//!     // Operation that might fail
//!     if false {
//!         Err(CalibornError::NotFound("Resource not found".to_string()))
//!     } else {
//!         Ok("Success".to_string())
//!     }
//! }
//! ```

use std::{borrow::Cow, fmt::Debug};

use axum::{
    extract::rejection::{JsonRejection, QueryRejection},
    http::{HeaderName, HeaderValue},
    response::IntoResponse,
};
use reqwest::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use crate::dtos::Json;

#[derive(Debug)]
pub struct PublicError {
    pub status: StatusCode,
    pub code: Cow<'static, str>,
    pub message: Cow<'static, str>,
    pub additional_headers: Vec<(HeaderName, HeaderValue)>,
}

impl PublicError {
    pub fn new(code: &'static str, message: &'static str, status: StatusCode) -> Self {
        Self {
            status,
            code: Cow::Borrowed(code),
            message: Cow::Borrowed(message),
            additional_headers: Vec::new(),
        }
    }

    pub fn with_owned<S1, S2>(code: S1, message: S2, status: StatusCode) -> Self
    where
        S1: Into<Cow<'static, str>>,
        S2: Into<Cow<'static, str>>,
    {
        Self {
            status,
            code: code.into(),
            message: message.into(),
            additional_headers: Vec::new(),
        }
    }

    pub fn with_header<K, V>(mut self, name: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        <K as TryInto<HeaderName>>::Error: Debug,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: Debug,
    {
        self.additional_headers
            .push((name.try_into().unwrap(), value.try_into().unwrap()));
        self
    }
}

/// Errors that may have a public HTTP representation.
pub trait ToPublicError {
    /// Convert this error to a possible public error.
    ///
    /// Returns `None` if the error is not public and is handled like a 500 Internal Server Error.
    /// Returns `Some` if the error is public and is handled as the public error is configured.
    fn as_public(&self) -> Option<PublicError>;
}

#[derive(Debug)]
pub enum ApiError {
    Public(PublicError),
    Internal(anyhow::Error),
}

impl<E> From<E> for ApiError
where
    E: std::error::Error + Send + Sync + ToPublicError + 'static,
{
    fn from(value: E) -> Self {
        match value.as_public() {
            Some(pub_) => ApiError::Public(pub_),
            None => ApiError::Internal(anyhow::Error::new(value)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::Public(p) => {
                let body = Json(ErrorResponse {
                    message: p.message.to_string(),
                    error: p.code.to_string(),
                });
                let mut res = (p.status, body).into_response();
                for (name, value) in p.additional_headers {
                    res.headers_mut().insert(name, value);
                }

                res
            }
            ApiError::Internal(e) => {
                tracing::error!(error = ?e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(value: JsonRejection) -> Self {
        Self::Public(PublicError::with_owned(
            "invalid-json",
            value.body_text(),
            value.status(),
        ))
    }
}

impl From<QueryRejection> for ApiError {
    fn from(value: QueryRejection) -> Self {
        Self::Public(PublicError::with_owned(
            "invalid-query-parameters",
            value.body_text(),
            value.status(),
        ))
    }
}

/// A Result type that returns either a value or a CalibornError.
///
/// This type alias simplifies error handling throughout the application by
/// providing a consistent result type.
///
/// # Examples
///
/// ```rust
/// # use caliborn::dtos::error::{CalibornError, CalibornResult};
/// fn fetch_user(id: u64) -> CalibornResult<String> {
///     // Simulate a database lookup
///     if id == 0 {
///         Err(CalibornError::NotFound("User not found".to_string()))
///     } else {
///         Ok("User data".to_string())
///     }
/// }
/// ```
pub type CalibornResult<T> = Result<T, ApiError>;

/// An error response containing details about the problem.
///
/// This struct is serialized and returned to the client when an error occurs.
/// It contains a human-readable message and a machine-readable error code.
///
/// # Examples
///
/// ```rust
/// # use caliborn::dtos::error::ErrorResponse;
/// let error = ErrorResponse {
///     message: String::from("Invalid authorization code"),
///     error: String::from("Unauthorized"),
/// };
/// ```
#[derive(Serialize, ToSchema)]
#[schema(
    description = "An error response containing details about the problem",
    examples(
        json!({
            "message": "Invalid authorization code",
            "error": "Unauthorized"
        })
    )
)]
pub struct ErrorResponse {
    /// A human-readable message describing the error.
    #[schema(examples("Invalid authorization code"))]
    pub message: String,

    /// A machine-readable error code identifying the error type.
    ///
    /// Common values include "Bad Request", "Unauthorized", "Internal Server Error",
    /// and "Not Found".
    #[schema(examples("Bad Request", "Unauthorized", "Internal Server Error"))]
    pub error: String,
}
