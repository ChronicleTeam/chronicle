//! Route handlers for managing user tables.
//!
//! Users must be authenticated for all requests.

mod entries;
mod fields;
mod tables;

use super::ApiState;
use axum::Router;

pub fn router() -> Router<ApiState> {
    Router::new()
        .merge(tables::router())
        .merge(fields::router())
        .merge(entries::router())
}
