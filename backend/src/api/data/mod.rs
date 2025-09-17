//! Route handlers for managing user tables.
//!
//! Users must be authenticated for all requests.

mod entries;
mod fields;
mod tables;

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(tables::router())
        .merge(fields::router())
        .merge(entries::router())
}