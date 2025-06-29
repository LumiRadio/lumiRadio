use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::dtos::json;

fn default_page_size() -> u64 {
    10
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: default_page_size(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64, total_pages: u64) -> Self {
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }

    pub fn map<F, U>(self, f: F) -> Page<U>
    where
        F: Fn(T) -> U,
    {
        Page {
            items: self.items.into_iter().map(f).collect(),
            total: self.total,
            page: self.page,
            page_size: self.page_size,
            total_pages: self.total_pages,
        }
    }
}

impl<T> IntoResponse for Page<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        json(self).into_response()
    }
}
