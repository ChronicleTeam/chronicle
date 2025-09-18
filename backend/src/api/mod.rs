mod data;
mod users;
mod viz;

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new().nest(
        "/api",
        Router::new()
            .merge(users::router())
            .merge(data::router())
            .merge(viz::router()),
    )
}
