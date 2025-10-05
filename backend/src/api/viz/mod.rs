//! Route handlers for managing user dashboards.
//!
//! Users must be authenticated for all requests.

mod axes;
mod charts;
mod dashboards;

use aide::axum::ApiRouter;

use crate::AppState;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .merge(dashboards::router())
        .merge(charts::router())
        .merge(axes::router())
}
