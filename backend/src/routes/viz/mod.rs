mod chart;
mod dashboard;
mod axis;

use super::ApiState;
use axum::Router;

pub(crate) fn router() -> Router<ApiState> {
    Router::new().merge(chart::router())
}
