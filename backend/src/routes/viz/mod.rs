//! Route handlers for managing user dashboards.
//!
//! Users must be authenticated for all requests.

mod axes;
mod charts;
mod dashboards;

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(dashboards::router())
        .merge(charts::router())
        .merge(axes::router())
}
