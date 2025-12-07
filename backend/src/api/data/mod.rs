//! Routes for managing user tables, fields, and entries.
//!
//! Users must have the appropriate access role for any operation.
//! Otherwise, `403 Forbidden` or `404 Not Found` is returned.

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
