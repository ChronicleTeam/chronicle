use axum::Router;

use super::ApiState;

mod plot;

pub(crate) fn router() -> Router<ApiState> {
    Router::new()
}