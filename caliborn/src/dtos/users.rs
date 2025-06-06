//! User-related data transfer objects for the Caliborn API.
//!
//! This module contains DTOs related to users in the lumiRadio system,
//! including user profiles and conversion functions from database entities.
//!
//! # Examples
//!
//! ```rust
//! # use caliborn::dtos::users::UserDto;
//! # use chrono::NaiveDateTime;
//! let user = UserDto {
//!     id: 675674657,
//!     watched_time: 1234567890,
//!     boonbucks: 100,
//!     username: Some(String::from("Caliborn")),
//!     last_message_sent: None,
//!     migrated: true,
//!     created_at: NaiveDateTime::parse_from_str("2023-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
//!     updated_at: NaiveDateTime::parse_from_str("2023-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
//! };
//! ```

use axum::response::IntoResponse;
use chrono::NaiveDateTime;
use serde::Serialize;
use utoipa::ToSchema;

use crate::entities;

use super::json;

/// A user of the lumiRadio system.
///
/// This DTO represents a user and their associated data, including statistics
/// like watched time and currency balance. It is used for API responses when
/// user information is requested.
///
/// # Examples
///
/// ```rust
/// # use caliborn::dtos::users::UserDto;
/// # use chrono::NaiveDateTime;
/// let user = UserDto {
///     id: 675674657,
///     watched_time: 1234567890,
///     boonbucks: 100,
///     username: Some(String::from("Caliborn")),
///     last_message_sent: None,
///     migrated: true,
///     created_at: NaiveDateTime::parse_from_str("2023-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
///     updated_at: NaiveDateTime::parse_from_str("2023-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
/// };
/// ```
#[derive(Serialize, ToSchema)]
#[schema(
    description = "A user of lumiRadio",
    examples(
        json!({
            "id": 675674657,
            "watched_time": 1234567890,
            "boonbucks": 100,
            "username": "Caliborn",
            "last_message_sent": "2023-01-01T00:00:00",
            "migrated": true,
            "created_at": "2023-01-01T00:00:00",
            "updated_at": "2023-01-01T00:00:00"
        })
    )
)]
pub struct UserDto {
    /// The unique identifier of the user.
    #[schema(example = 675674657)]
    pub id: i64,

    /// The total watched time of the user, in seconds.
    ///
    /// This represents the cumulative time the user has spent watching content
    /// on the lumiRadio platform.
    #[schema(example = 1234567890)]
    pub watched_time: i64,

    /// The amount of boonbucks (virtual currency) the user has accumulated.
    ///
    /// Boonbucks can be earned through various activities on the platform and
    /// can be used for purchases or interactions.
    #[schema(example = 100)]
    pub boonbucks: i32,

    /// The username of the user, if they have set one.
    ///
    /// This is `None` if the user has not set a username yet.
    #[schema(example = "Caliborn")]
    pub username: Option<String>,

    /// The timestamp of the last message sent by the user.
    ///
    /// This is `None` if the user has never sent a message.
    #[schema(example = "2023-01-01T00:00:00")]
    pub last_message_sent: Option<NaiveDateTime>,

    /// Whether the user has migrated their data from the old radio bot.
    ///
    /// This is `true` if the user has migrated their data from the old radio bot.
    #[schema(example = true)]
    pub migrated: bool,

    /// The timestamp when the user account was created.
    #[schema(example = "2023-01-01T00:00:00")]
    pub created_at: NaiveDateTime,

    /// The timestamp when the user account was last updated.
    #[schema(example = "2023-01-01T00:00:00")]
    pub updated_at: NaiveDateTime,
}

impl IntoResponse for UserDto {
    fn into_response(self) -> axum::response::Response {
        json(self).into_response()
    }
}

/// Implements conversion from a database entity to a DTO.
///
/// This implementation allows for easy conversion from the database model
/// to the API response type.
///
/// # Examples
///
/// ```rust
/// # use caliborn::dtos::users::UserDto;
/// # use caliborn::entities;
/// # use chrono::NaiveDateTime;
/// # struct MockModel {
/// #     id: i64,
/// #     watched_time: i64,
/// #     boonbucks: i32,
/// #     username: Option<String>,
/// #     last_message_sent: Option<NaiveDateTime>,
/// #     migrated: bool,
/// #     created_at: NaiveDateTime,
/// #     updated_at: NaiveDateTime,
/// # }
/// # impl From<MockModel> for entities::users::Model {
/// #     fn from(m: MockModel) -> Self { todo!() }
/// # }
/// # let db_model = MockModel {
/// #     id: 1,
/// #     watched_time: 3600,
/// #     boonbucks: 50,
/// #     username: Some(String::from("User1")),
/// #     last_message_sent: None,
/// #     migrated: false,
/// #     created_at: NaiveDateTime::parse_from_str("2023-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
/// #     updated_at: NaiveDateTime::parse_from_str("2023-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
/// # };
/// // When fetching a user from the database
/// // let db_model = fetch_user_from_db(user_id).await?;
///
/// // Convert to DTO for API response
/// // let user_dto = UserDto::from(db_model);
/// ```
impl From<entities::users::Model> for UserDto {
    fn from(value: entities::users::Model) -> Self {
        Self {
            id: value.id,
            watched_time: value.watched_time,
            boonbucks: value.boonbucks,
            username: value.username,
            last_message_sent: value.last_message_sent,
            migrated: value.migrated,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
