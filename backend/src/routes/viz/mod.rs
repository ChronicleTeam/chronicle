use axum::Router;

use super::ApiState;

mod chart;

pub(crate) fn router() -> Router<ApiState> {
    Router::new()
}