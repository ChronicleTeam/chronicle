mod access;
mod data;
mod users;
mod viz;

use crate::AppState;
use aide::axum::ApiRouter;

const NO_DATA_IN_REQUEST_BODY: &str = "No data in request body";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/api",
        ApiRouter::new()
            .merge(users::router())
            .merge(data::router())
            .merge(viz::router())
            .merge(access::router()),
    )
}
