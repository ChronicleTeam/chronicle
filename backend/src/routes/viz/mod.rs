mod charts;
mod dashboards;
mod axes;

use super::ApiState;
use axum::Router;

pub(crate) fn router() -> Router<ApiState> {
    Router::new().merge(dashboards::router()).merge(charts::router())
}
