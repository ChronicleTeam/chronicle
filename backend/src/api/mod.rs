mod data;
mod users;
mod viz;

use crate::AppState;
use aide::axum::ApiRouter;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/api",
        ApiRouter::new()
            .merge(users::router())
            .merge(data::router())
            .merge(viz::router()),
    )
}
