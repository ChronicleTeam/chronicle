mod users;
mod data;
mod viz;

use axum::Router;
use sqlx::PgPool;
use crate::{model::users::Credentials, AppState};


pub async fn router(db: PgPool, admin_creds: Credentials) -> anyhow::Result<Router<AppState>> {
    Ok(Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(users::router(db, admin_creds).await?)
                .merge(data::router())
                .merge(viz::router()),
        ))
}