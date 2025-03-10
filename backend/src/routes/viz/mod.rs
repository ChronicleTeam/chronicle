mod charts;
mod dashboards;

use super::ApiState;
use axum::Router;

pub(crate) fn router() -> Router<ApiState> {
    Router::new().merge(dashboards::router()).merge(charts::router())
}
