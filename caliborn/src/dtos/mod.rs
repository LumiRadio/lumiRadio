//! Data transfer objects for the Caliborn API.
//!
//! This module contains all the data structures used for communication
//! between the API and its clients. It includes request and response types,
//! JSON handling utilities, and error types.
//!
//! # Examples
//!
//! ```rust
//! # use caliborn::dtos::{json, json_with_status};
//! # use caliborn::dtos::error::CalibornResult;
//! # use reqwest::StatusCode;
//! # use serde::Serialize;
//! #[derive(Serialize)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//!
//! // Return a JSON response with 200 OK status
//! fn get_user() -> CalibornResult<axum::response::Response> {
//!     let user = User { id: 1, name: String::from("Alice") };
//!     json(user)
//! }
//!
//! // Return a JSON response with 201 Created status
//! fn create_user() -> CalibornResult<axum::response::Response> {
//!     let user = User { id: 2, name: String::from("Bob") };
//!     json_with_status(user, StatusCode::CREATED)
//! }
//! ```

use axum::{extract::FromRequest, response::IntoResponse};
use axum_macros::FromRequestParts;
use error::ApiError;
use reqwest::StatusCode;
use serde::{Serialize, de::DeserializeOwned};

use crate::dtos::error::CalibornResult;

pub mod auth;
pub mod cans;
pub mod error;
pub mod page;
pub mod songs;
pub mod users;

/// Return a JSON response with the provided value.
///
/// This is a convenience function that returns a JSON response with
/// the provided value. The response will have a status code of 200 OK.
///
/// # Examples
///
/// ```rust
/// # use caliborn::dtos::json;
/// # use caliborn::dtos::error::CalibornResult;
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// fn get_user() -> CalibornResult<axum::response::Response> {
///     let user = User { id: 1, name: String::from("Alice") };
///     json(user)
/// }
/// ```
///
/// # Errors
///
/// If the value cannot be serialized to JSON, this function will return
/// a `CalibornError`.
pub fn json<T: Serialize>(value: T) -> CalibornResult<axum::response::Response> {
    Ok(Json(value).into_response())
}

/// Return a JSON response with the provided value and status code.
///
/// This is a convenience function that returns a JSON response with
/// the provided value and status code. The response will have the
/// provided status code.
///
/// # Examples
///
/// ```rust
/// # use caliborn::dtos::json_with_status;
/// # use caliborn::dtos::error::CalibornResult;
/// # use reqwest::StatusCode;
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// fn create_user() -> CalibornResult<axum::response::Response> {
///     let user = User { id: 2, name: String::from("Bob") };
///     json_with_status(user, StatusCode::CREATED)
/// }
/// ```
///
/// # Errors
///
/// If the value cannot be serialized to JSON, this function will return
/// a `CalibornError`.
pub fn json_with_status<T: Serialize>(
    value: T,
    status: StatusCode,
) -> CalibornResult<axum::response::Response> {
    Ok((status, Json(value)).into_response())
}

/// A custom extractor that will deserialize JSON from the request body
/// and return the inner value with a custom error type.
///
/// This extractor is identical to the built-in `axum::Json` extractor,
/// but it returns a `CalibornError` instead of an `axum::JsonRejection`.
///
/// # Examples
///
/// ```rust
/// # use axum::{routing::post, Router};
/// # use caliborn::dtos::Json;
/// # use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
/// }
///
/// async fn create_user(Json(payload): Json<CreateUser>) {
///     // Process the deserialized JSON payload
///     let name = payload.name;
///     let email = payload.email;
///     // ...
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// ```
///
/// # Errors
///
/// If the request body cannot be deserialized as JSON, this extractor will
/// return a `CalibornError::JsonRejection`.
#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct Json<T>(pub T);

impl<T: DeserializeOwned> Json<T> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ApiError> {
        let axum_json = axum::Json::from_bytes(bytes)?;
        Ok(Json(axum_json.0))
    }
}

/// A custom extractor for query parameters.
///
/// This extractor is identical to the built-in `axum::extract::Query` extractor,
/// but it provides a custom error type (`CalibornError`) instead of the default
/// `axum::extract::QueryRejection`.
///
/// # Examples
///
/// ```rust
/// use axum::extract::Query;
/// use caliborn::dtos::Query;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UserQuery {
///     id: u64,
/// }
///
/// async fn get_user(Query(query): Query<UserQuery>) -> Json<User> {
///     Json(User {
///         id: query.id,
///         name: String::from("Alice"),
///     })
/// }
/// ```
#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct Query<T>(pub T);

/// Implementation of Axum's IntoResponse trait for `Json<T>`.
///
/// This implementation allows `Json<T>` to be directly returned from
/// Axum handler functions, automatically converting the value into a
/// JSON response.
///
/// # Examples
///
/// ```rust
/// # use axum::response::IntoResponse;
/// # use caliborn::dtos::Json;
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// async fn get_user() -> Json<User> {
///     Json(User {
///         id: 1,
///         name: String::from("Alice"),
///     })
/// }
/// ```
impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}
