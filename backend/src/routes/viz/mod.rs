mod axes;
mod charts;
mod dashboards;

use super::ApiState;
use axum::Router;

pub fn router() -> Router<ApiState> {
    Router::new()
        .merge(dashboards::router())
        .merge(charts::router())
        .merge(axes::router())
}
