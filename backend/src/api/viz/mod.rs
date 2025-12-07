//! Routes for managing user dashboards, charts, and axes.
//!
//! Users must have the appropriate access role for any operation.
//! Otherwise, `403 Forbidden` or `404 Not Found` is returned.

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
