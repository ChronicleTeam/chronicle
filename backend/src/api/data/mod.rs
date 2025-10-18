//! Route handlers for managing user tables.
//!
//! Users must be authenticated for all requests.

mod entries;
mod fields;
mod tables;

use crate::AppState;
use aide::axum::ApiRouter;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .merge(tables::router())
        .merge(fields::router())
        .merge(entries::router())
}
