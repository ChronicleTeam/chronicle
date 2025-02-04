use super::ApiState;
use crate::{
    error::ApiResult,
    model::{CreateTable, Table, TableId},
    query, Id,
};
use axum::{extract::State, routing::post, Json, Router};
use axum_macros::debug_handler;
use sqlx::{prelude::*, PgPool};

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables",
        Router::new().route("/", post(create_table).get(get_all_tables)),
    )
}

// #[debug_handler]
async fn create_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_table): Json<CreateTable>,
) -> ApiResult<Json<TableId>> {
    // TESTING
    let user_id = debug_get_user_id(&pool).await?;

    let table_id =
        query::insert_table(&pool, user_id, create_table.name, create_table.description).await?;

    Ok(Json(TableId { table_id }))
}

// #[debug_handler]
async fn get_all_tables(
    State(ApiState { pool, .. }): State<ApiState>,
) -> ApiResult<Json<Vec<Table>>> {
    todo!("Not implemented")
}

async fn debug_get_user_id(pool: &PgPool) -> Result<Id, sqlx::Error> {
    Ok(sqlx::query("SELECT user_id FROM app_user LIMIT 1;")
        .fetch_one(pool)
        .await?
        .get("user_id"))
}
