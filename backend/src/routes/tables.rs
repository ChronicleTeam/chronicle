use super::ApiState;
use crate::{
    error::{ApiError, ApiResult, OnConstraint},
    model::{CreateTable, Table, TableId, UpdateTable},
    query, Id,
};
use axum::{
    extract::{Path, State},
    routing::{post, put},
    Json, Router,
};
use axum_macros::debug_handler;
use sqlx::{PgPool, Row};

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables",
        Router::new()
            .route("/", post(create_table).get(get_user_tables))
            .route("/{table_id}", put(update_table).delete(delete_table)),
    )
}

// #[debug_handler]
async fn create_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_table): Json<CreateTable>,
) -> ApiResult<Json<TableId>> {
    // TESTING
    let user_id = debug_get_user_id(&pool).await?;

    let table_id = query::create_table(&pool, user_id, create_table.name, create_table.description)
        .await
        .on_constraint("meta_table_user_id_name_key", |_| {
            ApiError::unprocessable_entity([("table", "table name already used")])
        })?;

    Ok(Json(TableId { table_id }))
}

// #[debug_handler]
async fn get_user_tables(
    State(ApiState { pool, .. }): State<ApiState>,
) -> ApiResult<Json<Vec<Table>>> {
    // TESTING
    let user_id = debug_get_user_id(&pool).await?;
    let tables = query::get_user_tables(&pool, user_id).await?;
    Ok(Json(tables))
}

async fn update_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(update_table): Json<UpdateTable>,
) -> ApiResult<()> {
    query::update_table(&pool, table_id, update_table.name, update_table.description).await?;
    Ok(())
}

async fn delete_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<()> {
    query::delete_table(&pool, table_id).await?;
    Ok(())
}

async fn debug_get_user_id(pool: &PgPool) -> Result<Id, sqlx::Error> {
    let (user_id,): (Id,) = sqlx::query_as("SELECT user_id FROM app_user LIMIT 1;")
        .fetch_one(pool)
        .await?;
    Ok(user_id)
}
