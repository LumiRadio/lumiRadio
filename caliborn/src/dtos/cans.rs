use axum::response::IntoResponse;
use serde::Serialize;
use utoipa::ToSchema;

use crate::dtos::json;

#[derive(Serialize, ToSchema)]
#[schema(
    examples(json!({"count": 10}))
)]
pub struct CanCountDto {
    pub count: u64,
}

impl IntoResponse for CanCountDto {
    fn into_response(self) -> axum::response::Response {
        json(self).into_response()
    }
}
