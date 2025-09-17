mod users;
mod data;
mod viz;

use axum::Router;
use crate::AppState;


pub fn router() -> Router<AppState> {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(users::router())
                .merge(data::router())
                .merge(viz::router()),
        )
}